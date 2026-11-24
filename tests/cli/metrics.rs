use super::*;
use opentelemetry_proto::tonic::{
    collector::metrics::v1::ExportMetricsServiceRequest,
    common::v1::{KeyValue, any_value},
    metrics::v1::{Metric, NumberDataPoint, metric, number_data_point},
};
use prost::Message;
use tokio::sync::mpsc;

struct Collector {
    endpoint: String,
    stalled: Arc<std::sync::atomic::AtomicBool>,
    received: mpsc::Receiver<ExportMetricsServiceRequest>,
    task: tokio::task::JoinHandle<()>,
}
impl Collector {
    async fn new(stall: bool) -> Self {
        use paydirt_server::axum::{self, body::Bytes, http::HeaderMap, routing::post};
        let (send, received) = mpsc::channel(64);
        let stalled = Arc::new(std::sync::atomic::AtomicBool::new(stall));
        let blocked = stalled.clone();
        let router = axum::Router::new().route(
            "/custom/metrics",
            post(move |headers: HeaderMap, bytes: Bytes| {
                let send = send.clone();
                let blocked = blocked.clone();
                async move {
                    assert_eq!(
                        headers.get("content-type").unwrap(),
                        "application/x-protobuf"
                    );
                    let batch = ExportMetricsServiceRequest::decode(bytes).unwrap();
                    if blocked.load(Ordering::Relaxed) {
                        tokio::time::sleep(Duration::from_secs(60)).await;
                    } else {
                        let _ = send.try_send(batch);
                    }
                    (
                        [("content-type", "application/x-protobuf")],
                        Vec::<u8>::new(),
                    )
                }
            }),
        );
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!("http://{}/custom/metrics", listener.local_addr().unwrap());
        let task = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        Self {
            endpoint,
            stalled,
            received,
            task,
        }
    }
    fn config(&self) -> serde_json::Value {
        serde_json::json!({"endpoint": self.endpoint, "interval_ms": 100, "timeout_ms": 100})
    }
    async fn until(
        &mut self,
        condition: impl Fn(&ExportMetricsServiceRequest) -> bool,
    ) -> ExportMetricsServiceRequest {
        tokio::time::timeout(Duration::from_secs(15), async {
            loop {
                let batch = self.received.recv().await.expect("collector stopped");
                if condition(&batch) {
                    return batch;
                }
            }
        })
        .await
        .expect("expected OTLP metrics did not arrive")
    }
}
impl Drop for Collector {
    fn drop(&mut self) {
        self.task.abort();
    }
}

fn attribute<'a>(values: &'a [KeyValue], name: &str) -> Option<&'a str> {
    match values
        .iter()
        .find(|a| a.key == name)?
        .value
        .as_ref()?
        .value
        .as_ref()?
    {
        any_value::Value::StringValue(value) => Some(value),
        _ => None,
    }
}
fn resource<'a>(batch: &'a ExportMetricsServiceRequest, name: &str) -> Option<&'a str> {
    attribute(
        &batch
            .resource_metrics
            .first()?
            .resource
            .as_ref()?
            .attributes,
        name,
    )
}
fn metric<'a>(batch: &'a ExportMetricsServiceRequest, name: &str) -> Option<&'a Metric> {
    batch
        .resource_metrics
        .iter()
        .flat_map(|r| &r.scope_metrics)
        .flat_map(|s| &s.metrics)
        .find(|m| m.name == name)
}
fn points<'a>(batch: &'a ExportMetricsServiceRequest, name: &str) -> &'a [NumberDataPoint] {
    match metric(batch, name).and_then(|m| m.data.as_ref()) {
        Some(metric::Data::Gauge(gauge)) => &gauge.data_points,
        Some(metric::Data::Sum(sum)) => &sum.data_points,
        _ => &[],
    }
}
fn number(point: &NumberDataPoint) -> f64 {
    match point.value.unwrap() {
        number_data_point::Value::AsInt(value) => value as f64,
        number_data_point::Value::AsDouble(value) => value,
    }
}
fn count(batch: &ExportMetricsServiceRequest, phase: &str) -> f64 {
    points(batch, "wafl.runner.instances")
        .iter()
        .find(|p| attribute(&p.attributes, "wafl.instance.phase") == Some(phase))
        .map(number)
        .unwrap_or(-1.0)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn otlp_exports_instances_and_uses_a_new_session_after_restart() {
    let directory = tempfile::tempdir().unwrap();
    let assets = Assets::new(false).await;
    let mut collector = Collector::new(false).await;
    let mut server = Server::configured(directory.path(), false, Some(collector.config())).await;
    let initial = collector
        .until(|batch| count(batch, "running") == 0.0)
        .await;
    assert_eq!(resource(&initial, "service.name"), Some("wafl-runner"));
    let session = resource(&initial, "service.instance.id")
        .unwrap()
        .to_owned();
    assert!(uuid::Uuid::parse_str(&session).is_ok());
    assert_eq!(metric(&initial, "wafl.runner.uptime").unwrap().unit, "s");
    #[cfg(target_os = "linux")]
    {
        assert!(number(&points(&initial, "process.memory.usage")[0]) > 0.0);
        let Some(metric::Data::Sum(cpu)) = &metric(&initial, "process.cpu.time").unwrap().data
        else {
            panic!("CPU must be a cumulative sum");
        };
        assert!(cpu.is_monotonic);
        assert_eq!(cpu.aggregation_temporality, 2);
        assert!(number(&cpu.data_points[0]) >= 0.0);
    }
    let app = instance(
        &server.endpoint,
        &["add", &assets.url, "--alias", "observed"],
    )
    .await;
    assert_eq!(paydirt::format_id(&app.session).unwrap(), session);
    let id = paydirt::format_id(&app.id).unwrap();
    let active = collector
        .until(|batch| count(batch, "running") == 1.0)
        .await;
    assert_eq!(number(&points(&active, "wafl.runner.executions")[0]), 1.0);
    assert_eq!(
        number(&points(&active, "wafl.runner.memory.reserved")[0]),
        67108864.0
    );
    assert_eq!(number(&points(&active, "wafl.runner.compilations")[0]), 1.0);
    assert_eq!(number(&points(&active, "wafl.runner.accepting")[0]), 1.0);
    assert!(metric(&active, "wafl.runner.compilation.requests").is_some());
    assert!(
        points(&active, "wafl.instance.active")
            .iter()
            .any(
                |p| attribute(&p.attributes, "wafl.instance.id") == Some(id.as_str())
                    && number(p) == 0.0
            )
    );
    assert_eq!(
        server.get(&app, "/data").await.text().await.unwrap(),
        "shared data"
    );
    instance(&server.endpoint, &["stop", "@observed"]).await;
    collector
        .until(|batch| count(batch, "stopped") == 1.0 && count(batch, "running") == 0.0)
        .await;
    success(cli(&server.endpoint, &["instance", "erase", "@observed"]).await);
    collector
        .until(|batch| {
            count(batch, "stopped") == 0.0 && points(batch, "wafl.instance.active").is_empty()
        })
        .await;
    #[cfg(unix)]
    {
        // A long interval makes the next update prove that graceful shutdown
        // performs a final export, rather than waiting for the next timer tick.
        server.stop().await;
        let mut config = collector.config();
        config["interval_ms"] = 60_000.into();
        server = Server::configured(directory.path(), false, Some(config)).await;
        assert!(
            Command::new("kill")
                .args(["-TERM", &server.child.id().unwrap().to_string()])
                .status()
                .await
                .unwrap()
                .success()
        );
        assert!(
            tokio::time::timeout(Duration::from_secs(10), server.child.wait())
                .await
                .unwrap()
                .unwrap()
                .success()
        );
        let final_batch = collector
            .until(|batch| resource(batch, "service.instance.id") != Some(session.as_str()))
            .await;
        assert_eq!(count(&final_batch, "running"), 0.0);
    }
    server.stop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn otlp_timeout_does_not_block_apps_and_export_recovers() {
    let directory = tempfile::tempdir().unwrap();
    let assets = Assets::new(false).await;
    let mut collector = Collector::new(true).await;
    let mut server = Server::configured(directory.path(), false, Some(collector.config())).await;
    let app = instance(&server.endpoint, &["add", &assets.url]).await;
    let body = tokio::time::timeout(Duration::from_secs(3), async {
        server.get(&app, "/data").await.text().await.unwrap()
    })
    .await
    .expect("stalled metrics export blocked app traffic");
    assert_eq!(body, "shared data");
    collector.stalled.store(false, Ordering::Relaxed);
    let batch = collector
        .until(|batch| {
            points(batch, "wafl.runner.metrics.failures")
                .first()
                .is_some_and(|p| number(p) > 0.0)
        })
        .await;
    assert_eq!(count(&batch, "running"), 1.0);
    collector.stalled.store(true, Ordering::Relaxed);
    #[cfg(unix)]
    {
        assert!(
            Command::new("kill")
                .args(["-TERM", &server.child.id().unwrap().to_string()])
                .status()
                .await
                .unwrap()
                .success()
        );
        assert!(
            tokio::time::timeout(Duration::from_secs(10), server.child.wait())
                .await
                .unwrap()
                .unwrap()
                .success()
        );
    }
    server.stop().await;
}

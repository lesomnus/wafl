use crate::config::Metrics;
use anyhow::{Context, Result};
use opentelemetry::{KeyValue, metrics::MeterProvider};
use opentelemetry_otlp::{MetricExporter, Protocol, WithExportConfig, WithHttpConfig};
use opentelemetry_sdk::{
    Resource,
    error::OTelSdkResult,
    metrics::{
        PeriodicReader, SdkMeterProvider, Temporality, data::ResourceMetrics,
        exporter::PushMetricExporter,
    },
};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

pub(crate) struct InstanceSample {
    pub id: String,
    pub phase: &'static str,
    pub active: u64,
}
pub(crate) struct Snapshot {
    pub instances: Vec<InstanceSample>,
    pub pending: u64,
    pub preparing: u64,
    pub usage: crate::proto::wafl::Usage,
    pub compilations: u64,
    pub compile_hits: u64,
    pub compile_misses: u64,
    pub accepting: bool,
}
pub(crate) type Observe = Arc<dyn Fn() -> Option<Snapshot> + Send + Sync>;

/// Owned by the server, with weak references back to runner state. Each server
/// has its own provider; it never changes OpenTelemetry's process-global one.
pub(crate) struct Telemetry(Option<SdkMeterProvider>);
impl Telemetry {
    pub async fn start(config: Metrics, observe: Observe, session: String) -> Result<Self> {
        tokio::task::spawn_blocking(move || Self::build(config, observe, session))
            .await
            .context("initialize metrics worker")?
    }

    fn build(config: Metrics, observe: Observe, session: String) -> Result<Self> {
        let timeout = Duration::from_millis(config.timeout_ms);
        let client = reqwest::blocking::Client::builder()
            .timeout(timeout)
            .connect_timeout(timeout)
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        let exporter = MetricExporter::builder()
            .with_http()
            .with_http_client(client)
            .with_protocol(Protocol::HttpBinary)
            .with_endpoint(config.endpoint)
            .with_timeout(timeout)
            .with_temporality(Temporality::Cumulative)
            .build()?;
        let failures = Arc::new(AtomicU64::new(0));
        let reader = PeriodicReader::builder(Exporter {
            inner: exporter,
            failures: failures.clone(),
            warning: Mutex::new(None),
        })
        .with_interval(Duration::from_millis(config.interval_ms))
        .build();
        let resource = Resource::builder_empty()
            .with_attributes([
                KeyValue::new("service.name", "wafl-runner"),
                KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                KeyValue::new("service.instance.id", session),
            ])
            .build();
        let provider = SdkMeterProvider::builder()
            .with_resource(resource)
            .with_reader(reader)
            .build();
        let meter = provider.meter("wafl.runner");
        let phases = observe.clone();
        meter
            .u64_observable_gauge("wafl.runner.instances")
            .with_description("Published instances by execution phase; excludes pending Add")
            .with_unit("{instance}")
            .with_callback(move |observer| {
                if let Some(snapshot) = phases() {
                    for phase in ["stopped", "starting", "running", "failed", "stopping"] {
                        let count = snapshot
                            .instances
                            .iter()
                            .filter(|i| i.phase == phase)
                            .count();
                        observer
                            .observe(count as u64, &[KeyValue::new("wafl.instance.phase", phase)]);
                    }
                }
            })
            .build();
        let pending = observe.clone();
        meter
            .u64_observable_gauge("wafl.runner.additions.pending")
            .with_description("Unpublished Instance IDs reserved by Add")
            .with_unit("{instance}")
            .with_callback(move |observer| {
                if let Some(snapshot) = pending() {
                    observer.observe(snapshot.pending, &[]);
                }
            })
            .build();
        let preparing = observe.clone();
        meter
            .u64_observable_gauge("wafl.runner.additions.active")
            .with_description("Occupied Add slots, including unfinished cancelled compilation")
            .with_unit("{operation}")
            .with_callback(move |observer| {
                if let Some(snapshot) = preparing() {
                    observer.observe(snapshot.preparing, &[]);
                }
            })
            .build();
        for (name, unit, select) in [
            (
                "wafl.runner.startups",
                "{instance}",
                (|s: &Snapshot| s.usage.startups) as fn(&Snapshot) -> u64,
            ),
            ("wafl.runner.executions", "{instance}", |s: &Snapshot| {
                s.usage.running
            }),
            ("wafl.runner.memory.reserved", "By", |s: &Snapshot| {
                s.usage.memory_bytes
            }),
            ("wafl.runner.requests", "{lease}", |s: &Snapshot| {
                s.usage.requests
            }),
            ("wafl.runner.waiting", "{instance}", |s: &Snapshot| {
                s.usage.waiting
            }),
            ("wafl.runner.compilations", "{component}", |s: &Snapshot| {
                s.compilations
            }),
            ("wafl.runner.accepting", "1", |s: &Snapshot| {
                u64::from(s.accepting)
            }),
        ] {
            let sample = observe.clone();
            meter
                .u64_observable_gauge(name)
                .with_unit(unit)
                .with_callback(move |observer| {
                    if let Some(snapshot) = sample() {
                        observer.observe(select(&snapshot), &[]);
                    }
                })
                .build();
        }
        let compilation = observe.clone();
        meter
            .u64_observable_counter("wafl.runner.compilation.requests")
            .with_unit("{operation}")
            .with_callback(move |observer| {
                if let Some(snapshot) = compilation() {
                    observer.observe(snapshot.compile_hits, &[KeyValue::new("result", "reused")]);
                    observer.observe(
                        snapshot.compile_misses,
                        &[KeyValue::new("result", "compiled")],
                    );
                }
            })
            .build();
        meter
            .u64_observable_gauge("wafl.instance.active")
            .with_description(
                "Execution leases: requests including startup waiters, plus Add/Start waiters",
            )
            .with_unit("{lease}")
            .with_callback(move |observer| {
                if let Some(snapshot) = observe() {
                    for instance in snapshot.instances {
                        observer.observe(
                            instance.active,
                            &[KeyValue::new("wafl.instance.id", instance.id)],
                        );
                    }
                }
            })
            .build();
        meter
            .u64_observable_counter("wafl.runner.metrics.failures")
            .with_description("Failed OTLP export attempts in this runner telemetry session")
            .with_unit("{attempt}")
            .with_callback(move |observer| observer.observe(failures.load(Ordering::Relaxed), &[]))
            .build();
        let start = Instant::now();
        meter
            .f64_observable_gauge("wafl.runner.uptime")
            .with_unit("s")
            .with_callback(move |observer| observer.observe(start.elapsed().as_secs_f64(), &[]))
            .build();
        #[cfg(target_os = "linux")]
        {
            meter
                .f64_observable_counter("process.cpu.time")
                .with_description("User and system CPU time of the entire runner process")
                .with_unit("s")
                .with_callback(|observer| {
                    if let Some(sample) = process() {
                        observer.observe(sample.0, &[]);
                    }
                })
                .build();
            meter.u64_observable_gauge("process.memory.usage")
                .with_description("Resident memory of the entire runner process, not a guest limit or cgroup usage")
                .with_unit("By")
                .with_callback(|observer| {
                    if let Some(sample) = process() { observer.observe(sample.1, &[]); }
                }).build();
        }
        Ok(Self(Some(provider)))
    }

    pub async fn shutdown(mut self) {
        if let Some(provider) = self.0.take() {
            match tokio::task::spawn_blocking(move || provider.shutdown()).await {
                Ok(Ok(())) => {}
                result => eprintln!("OTLP metrics shutdown: {result:?}"),
            }
        }
    }
}
impl Drop for Telemetry {
    fn drop(&mut self) {
        // Cancellation/unwinding must not run a blocking SDK shutdown on the
        // async executor. Normal server shutdown awaits the final export above.
        if let Some(provider) = self.0.take() {
            let _ = std::thread::Builder::new()
                .name("wafl-metrics-shutdown".into())
                .spawn(move || {
                    let _ = provider.shutdown();
                });
        }
    }
}

struct Exporter {
    inner: MetricExporter,
    failures: Arc<AtomicU64>,
    warning: Mutex<Option<Instant>>,
}
impl PushMetricExporter for Exporter {
    async fn export(&self, metrics: &ResourceMetrics) -> OTelSdkResult {
        let result = self.inner.export(metrics).await;
        if let Err(error) = &result {
            self.failures.fetch_add(1, Ordering::Relaxed);
            let mut warning = self.warning.lock().unwrap();
            if warning.is_none_or(|last| last.elapsed() >= Duration::from_secs(60)) {
                eprintln!("OTLP metrics export failed: {error}");
                *warning = Some(Instant::now());
            }
        }
        result
    }
    fn force_flush(&self) -> OTelSdkResult {
        self.inner.force_flush()
    }
    fn shutdown_with_timeout(&self, timeout: Duration) -> OTelSdkResult {
        self.inner.shutdown_with_timeout(timeout)
    }
    fn temporality(&self) -> Temporality {
        self.inner.temporality()
    }
}

#[cfg(target_os = "linux")]
fn process() -> Option<(f64, u64)> {
    let stat = std::fs::read_to_string("/proc/self/stat").ok()?;
    // SAFETY: sysconf reads process-independent constants and takes no pointers.
    let ticks = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    let page = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    if ticks <= 0 || page <= 0 {
        return None;
    }
    parse_process(&stat, ticks as f64, page as u64)
}
#[cfg(target_os = "linux")]
fn parse_process(stat: &str, ticks: f64, page: u64) -> Option<(f64, u64)> {
    // comm may contain spaces and parentheses. Fields after the final ')' start
    // at field 3 (state); utime/stime/rss are fields 14/15/24 respectively.
    let fields: Vec<_> = stat
        .get(stat.rfind(')')? + 1..)?
        .split_whitespace()
        .collect();
    let user = fields.get(11)?.parse::<u64>().ok()?;
    let system = fields.get(12)?.parse::<u64>().ok()?;
    let rss = fields.get(21)?.parse::<u64>().ok()?;
    Some((
        (user.checked_add(system)? as f64) / ticks,
        rss.checked_mul(page)?,
    ))
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    #[test]
    fn process_fields_allow_spaces_and_parentheses_in_command() {
        let mut fields = vec!["0"; 22];
        fields[0] = "R";
        fields[11] = "150";
        fields[12] = "50";
        fields[21] = "10";
        let stat = format!("123 (a ) b (worker)) {}", fields.join(" "));
        assert_eq!(super::parse_process(&stat, 100.0, 4096), Some((2.0, 40960)));
        assert_eq!(super::parse_process("invalid", 100.0, 4096), None);
        assert!(super::process().is_some());
    }
}

//! Optional Kubernetes input provider. The API server owns desired spec; WAFL
//! owns scheduling and execution. No Kubernetes access occurs on request paths.
mod state;
#[cfg(test)]
mod tests;

use super::*;
use config::KubernetesConfig;
use futures_util::StreamExt;
use kube::{
    Api, Client, ResourceExt,
    api::{Patch, PatchParams},
    core::{ApiResource, DynamicObject, GroupVersionKind},
    runtime::{Controller, controller::Action, watcher},
};
use paydirt::spin::Stop;
use serde::Deserialize;
use sha2::{Digest, Sha256};

const GROUP: &str = "wafl.opt.td";
const FINALIZER: &str = "wafl.opt.td/app";

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Spec {
    #[serde(default)]
    name: String,
    domain: String,
    manifest: Manifest,
    replicas: Option<u32>,
    scaling: Option<Scaling>,
}

#[derive(Clone)]
struct Owner {
    cluster: String,
    namespace: String,
    name: String,
    uid: String,
    tenant: Vec<u8>,
    generation: u64,
}
impl Owner {
    fn read(object: &DynamicObject, config: &KubernetesConfig) -> Result<Self, Error> {
        let namespace = object
            .namespace()
            .ok_or_else(|| Error("namespace missing".into()))?;
        let tenant = config
            .namespaces
            .get(&namespace)
            .ok_or_else(|| Error("namespace is not configured".into()))?;
        let uid = object
            .uid()
            .filter(|uid| uuid::Uuid::parse_str(uid).is_ok())
            .ok_or_else(|| Error("invalid Kubernetes UID".into()))?;
        let generation = object
            .metadata
            .generation
            .filter(|n| *n > 0)
            .ok_or_else(|| Error("generation missing".into()))? as u64;
        Ok(Self {
            cluster: config.cluster.clone(),
            namespace,
            name: object.name_any(),
            uid,
            tenant: paydirt::parse_id(tenant)
                .ok_or_else(|| Error("invalid tenant".into()))?
                .to_vec(),
            generation,
        })
    }
    fn id(&self, domain: u8) -> Vec<u8> {
        let mut hash = Sha256::new();
        for value in [GROUP, &self.cluster, &self.uid] {
            hash.update((value.len() as u64).to_be_bytes());
            hash.update(value.as_bytes());
        }
        let mut id = hash.finalize()[..16].to_vec();
        id[6] = (id[6] & 0x0f) | 0x80;
        id[8] = 0x80;
        id[9] = domain;
        id
    }
}

#[derive(Debug)]
struct Error(String);
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Error {}
impl From<kube::Error> for Error {
    fn from(e: kube::Error) -> Self {
        Self(e.to_string())
    }
}
impl From<ConnectError> for Error {
    fn from(e: ConnectError) -> Self {
        Self(
            e.message
                .unwrap_or_else(|| "controller operation failed".into()),
        )
    }
}

struct Context {
    controller: Control,
    config: KubernetesConfig,
    api: Api<DynamicObject>,
}

pub(super) struct Provider {
    client: Client,
    config: KubernetesConfig,
}
impl Provider {
    pub(super) async fn open(controller: &Control) -> anyhow::Result<Option<Self>> {
        let Some(config) = controller.config.providers.kubernetes.clone() else {
            return Ok(None);
        };
        let options = kube::config::KubeConfigOptions {
            context: config.context.clone(),
            ..Default::default()
        };
        let mut connection = if let Some(path) = &config.kubeconfig {
            kube::Config::from_custom_kubeconfig(
                kube::config::Kubeconfig::read_from(path)?,
                &options,
            )
            .await?
        } else if config.context.is_some() {
            kube::Config::from_kubeconfig(&options).await?
        } else {
            kube::Config::infer().await?
        };
        connection.connect_timeout = Some(Duration::from_secs(10));
        connection.read_timeout = Some(Duration::from_secs(310));
        let client = Client::try_from(connection)?;
        let tenants: Vec<_> = config.namespaces.values().cloned().collect();
        controller
            .db
            .transaction(move |tx| {
                for tenant in tenants {
                    call(
                        tx,
                        "Tenant",
                        "Get",
                        json!({"ref":{"id":encode(&paydirt::parse_id(&tenant).unwrap())}}),
                    )?;
                }
                Ok(())
            })
            .await
            .map_err(|e| anyhow::anyhow!("Kubernetes tenant binding: {e}"))?;
        Ok(Some(Self { client, config }))
    }
    pub(super) async fn run(
        self: Arc<Self>,
        controller: Control,
        stop: Stop,
    ) -> Result<(), String> {
        let resource = ApiResource::from_gvk(&GroupVersionKind::gvk(GROUP, "v1alpha1", "App"));
        let mut tasks = tokio::task::JoinSet::new();
        for namespace in self.config.namespaces.keys() {
            let api =
                Api::<DynamicObject>::namespaced_with(self.client.clone(), namespace, &resource);
            let context = Arc::new(Context {
                controller: controller.clone(),
                config: self.config.clone(),
                api: api.clone(),
            });
            let stop = stop.clone();
            let resource = resource.clone();
            tasks.spawn(async move {
                let stream = Controller::new_with(api, watcher::Config::default(), resource)
                    .graceful_shutdown_on(async move { stop.cancelled().await })
                    .run(reconcile, error_policy, context);
                tokio::pin!(stream);
                while let Some(result) = stream.next().await {
                    if let Err(error) = result {
                        eprintln!("wafl-kubernetes watch/reconcile: {error}");
                    }
                }
            });
        }
        while let Some(result) = tasks.join_next().await {
            result.map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

fn error_policy(object: Arc<DynamicObject>, error: &Error, context: Arc<Context>) -> Action {
    eprintln!(
        "wafl-kubernetes {}/{}: {error}",
        object.namespace().unwrap_or_default(),
        object.name_any()
    );
    Action::requeue(Duration::from_millis(context.config.interval_ms.max(5000)))
}

async fn finalizers(
    context: &Context,
    object: &DynamicObject,
    values: Vec<String>,
) -> Result<(), Error> {
    patched(
        context
            .api
            .patch(
                &object.name_any(),
                &PatchParams::default(),
                &Patch::Merge(json!({
                    "metadata":{"resourceVersion":object.resource_version(),"finalizers":values}
                })),
            )
            .await,
    )
}
fn patched(result: Result<DynamicObject, kube::Error>) -> Result<(), Error> {
    match result {
        Ok(_) => Ok(()),
        // Another replica or a user changed/deleted the object. The next watch
        // event or periodic reconciliation will read its current version.
        Err(kube::Error::Api(e)) if matches!(e.code, 404 | 409) => Ok(()),
        Err(error) => Err(error.into()),
    }
}
async fn reconcile(event: Arc<DynamicObject>, context: Arc<Context>) -> Result<Action, Error> {
    // Re-read instead of applying a queued watch snapshot after a newer event.
    let Some(object) = context.api.get_opt(&event.name_any()).await? else {
        return Ok(Action::await_change());
    };
    if object.uid() != event.uid() {
        return Ok(Action::await_change());
    }
    let owner = Owner::read(&object, &context.config)?;
    let mut owned = object.finalizers().to_vec();
    let registered = owned.iter().any(|v| v == FINALIZER);
    let again = Action::requeue(Duration::from_millis(context.config.interval_ms));
    if object.metadata.deletion_timestamp.is_some() {
        if !registered {
            return Ok(Action::await_change());
        }
        if state::release(&context.controller, owner.clone()).await? {
            owned.retain(|v| v != FINALIZER);
            finalizers(&context, &object, owned).await?;
            return Ok(again);
        }
        publish(
            &context,
            &object,
            Some(state::snapshot(&context.controller, owner).await?),
            Some("Deleting: waiting for gateway drain and instance release"),
        )
        .await?;
        return Ok(again);
    }
    if !registered {
        owned.push(FINALIZER.into());
        finalizers(&context, &object, owned).await?;
        return Ok(again);
    }
    let spec = serde_json::from_value::<Spec>(object.data["spec"].clone());
    let applied = match spec {
        Ok(spec) => state::apply(&context.controller, owner.clone(), spec)
            .await
            .map_err(Error::from),
        Err(_) => Err(Error("Invalid spec: use the App CRD schema".into())),
    };
    match applied {
        Ok(()) => {
            let snapshot = state::snapshot(&context.controller, owner).await?;
            publish(&context, &object, Some(snapshot), None).await?;
        }
        Err(error) => publish(&context, &object, None, Some(&error.0)).await?,
    }
    Ok(again)
}

async fn publish(
    context: &Context,
    object: &DynamicObject,
    snapshot: Option<state::Snapshot>,
    error: Option<&str>,
) -> Result<(), Error> {
    let previous = &object.data["status"];
    let generation = object.metadata.generation.unwrap_or(0);
    let (mut status, accepted, mut ready, mut reason, mut message) = if let Some(s) = snapshot {
        let desired = scaling::desired(&s.app);
        let ready = s.ready >= desired && s.error.is_empty();
        (
            json!({"app":paydirt::format_id(&s.app.id).unwrap(),"alias":s.app.alias,"observedGeneration":s.generation,
            "desired":desired,"ready":s.ready,"revision":s.app.revision}),
            true,
            ready,
            if !s.error.is_empty() {
                "ExecutionFailed"
            } else if desired == 0 {
                "ScaledToZero"
            } else if ready {
                "Running"
            } else {
                "Provisioning"
            },
            s.error,
        )
    } else {
        let deleting = object.metadata.deletion_timestamp.is_some();
        let mut status = if previous.is_object() {
            previous.clone()
        } else {
            json!({})
        };
        status["observedGeneration"] = json!(previous["observedGeneration"].as_u64().unwrap_or(0));
        (
            status,
            false,
            false,
            if deleting {
                "Deleting"
            } else {
                "ReconcileFailed"
            },
            error
                .unwrap_or("reconciliation failed")
                .chars()
                .take(512)
                .collect::<String>(),
        )
    };
    if object.metadata.deletion_timestamp.is_some() {
        ready = false;
        reason = "Deleting";
        message = "Waiting for gateway drain and instance release".into();
    }
    let mut conditions = Vec::new();
    for (kind, value) in [("Accepted", accepted), ("Ready", ready)] {
        let value = if value { "True" } else { "False" };
        let old = previous["conditions"]
            .as_array()
            .and_then(|cs| cs.iter().find(|c| c["type"] == kind));
        let date = old
            .filter(|c| c["status"] == value && c["reason"] == reason && c["message"] == message)
            .and_then(|c| c["lastTransitionTime"].as_str())
            .map(str::to_owned)
            .unwrap_or_else(|| k8s_openapi::jiff::Timestamp::now().to_string());
        conditions.push(json!({"type":kind,"status":value,"observedGeneration":generation,"reason":reason,"message":message,"lastTransitionTime":date}));
    }
    status["conditions"] = json!(conditions);
    if status == *previous {
        return Ok(());
    }
    patched(
        context
            .api
            .patch_status(
                &object.name_any(),
                &PatchParams::default(),
                &Patch::Merge(json!({
                    "metadata":{"resourceVersion":object.resource_version()},"status":status
                })),
            )
            .await,
    )
}

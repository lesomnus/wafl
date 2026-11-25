use super::*;
use paydirt_store::{Fault, Transaction};

fn source(tx: &mut Transaction<'_>, owner: &Owner) -> paydirt_store::Result<AppSource> {
    let id = owner.id(14);
    let page = call(
        tx,
        "AppSource",
        "List",
        json!({"filters":[{"ref":{"id":encode(&id)}}]}),
    )?;
    let record: AppSource = if let Some(row) = page["items"].as_array().and_then(|v| v.first()) {
        decode(row.clone())?
    } else {
        decode(call(
            tx,
            "AppSource",
            "Add",
            json!({"id":encode(&id),"tenant":{"id":encode(&owner.tenant)},
            "provider":"kubernetes","cluster":owner.cluster,"namespace":owner.namespace,"name":owner.name,"uid":owner.uid}),
        )?)?
    };
    if record.provider != "kubernetes"
        || record.cluster != owner.cluster
        || record.namespace != owner.namespace
        || record.name != owner.name
        || record.uid != owner.uid
        || record.tenant.id != owner.tenant
    {
        return Err(Fault::precondition(
            "Kubernetes source identity or tenant binding changed",
        ));
    }
    Ok(record)
}
fn find(tx: &mut Transaction<'_>, owner: &Owner) -> paydirt_store::Result<Option<App>> {
    let page = call(
        tx,
        "App",
        "List",
        json!({"filters":[{"ref":{"id":encode(&owner.id(10))}}]}),
    )?;
    let app: Option<App> = page["items"]
        .as_array()
        .and_then(|v| v.first())
        .cloned()
        .map(decode)
        .transpose()?;
    if app
        .as_ref()
        .is_some_and(|a| a.source.id != owner.id(14) || a.tenant.id != owner.tenant)
    {
        return Err(Fault::precondition("App is owned by another source"));
    }
    Ok(app)
}

pub(super) async fn apply(
    controller: &Control,
    owner: Owner,
    mut spec: Spec,
) -> Result<(), ConnectError> {
    let config = controller.config.clone();
    controller.db.transaction(move|tx| {
        let record=source(tx,&owner)?;
        if record.deleting || record.released {return Err(Fault::precondition("Kubernetes source was deleted"));}
        if owner.generation<record.generation {return Err(Fault::aborted("stale Kubernetes generation"));}
        let old=find(tx,&owner)?;
        if owner.generation==record.generation && old.is_some() {return Ok(());}
        if spec.manifest.port==0 {spec.manifest.port=8080;}
        let replicas=spec.replicas.unwrap_or(if spec.scaling.is_some() {0} else {1});
        let name=if spec.name.is_empty() {owner.name.clone()} else {spec.name};
        if let Some(old)=old {
            if old.domain!=spec.domain {return Err(Fault::invalid("domain is immutable; create a new App"));}
            // Preserve the bundle pinned by the first successful execution when
            // the declaration still uses the same URL without an explicit digest.
            if spec.manifest.bundle.digest.is_empty() && spec.manifest.bundle.kind==old.manifest.bundle.kind
                && let Some(bundle)=spec.manifest.bundle.as_option_mut() {
                bundle.digest=old.manifest.bundle.digest.clone();
            }
            let updated=decode(call(tx,"App","Patch",json!({"ref":{"id":encode(&old.id)},
                "name":name,"replicas":replicas,"manifest":spec.manifest,"scalingNull":spec.scaling.is_none(),"scaling":spec.scaling}))?)?;
            api::finish_patch(tx,old,updated,&config)?;
        } else {
            let id=owner.id(10);
            api::insert(tx,json!({"id":encode(&id),"tenant":{"id":encode(&owner.tenant)},
                "alias":format!("k8s-{}",paydirt::format_id(&id).unwrap()),"source":{"id":encode(&record.id)},
                "name":name,"domain":spec.domain,"replicas":replicas,"manifest":spec.manifest,"scaling":spec.scaling}),&config)?;
        }
        call(tx,"AppSource","Patch",json!({"ref":{"id":encode(&record.id)},"generation":owner.generation.to_string()}))?;
        Ok(())
    }).await
}

pub(super) async fn release(controller: &Control, owner: Owner) -> Result<bool, ConnectError> {
    let config = controller.config.clone();
    controller
        .db
        .transaction(move |tx| {
            let record = source(tx, &owner)?;
            if record.released {
                return Ok(true);
            }
            call(
                tx,
                "AppSource",
                "Patch",
                json!({"ref":{"id":encode(&record.id)},"deleting":true}),
            )?;
            if let Some(old) = find(tx, &owner)? {
                let app = if old.scaling.as_option().is_some() || old.replicas != 0 {
                    let updated = decode(call(
                        tx,
                        "App",
                        "Patch",
                        json!({"ref":{"id":encode(&old.id)},"scalingNull":true,"replicas":0}),
                    )?)?;
                    api::finish_patch(tx, old, updated, &config)?
                } else {
                    old
                };
                let placements = all::<Placement>(tx, "Placement")?;
                if placements
                    .iter()
                    .any(|p| p.app.id == app.id && p.phase != PlacementPhase::Released)
                {
                    return Ok(false);
                }
                let traffic = all::<Traffic>(tx, "Traffic")?;
                if traffic
                    .iter()
                    .any(|t| t.app.id == app.id && (!t.closed || t.active != 0))
                {
                    return Ok(false);
                }
                api::erase(tx, app)?;
            }
            call(
                tx,
                "AppSource",
                "Patch",
                json!({"ref":{"id":encode(&record.id)},"released":true}),
            )?;
            Ok(true)
        })
        .await
}

pub(super) struct Snapshot {
    pub app: App,
    pub generation: u64,
    pub ready: u32,
    pub error: String,
}
pub(super) async fn snapshot(controller: &Control, owner: Owner) -> Result<Snapshot, ConnectError> {
    let fresh = controller.config.fresh_ms;
    controller
        .db
        .transaction(move |tx| {
            let record = source(tx, &owner)?;
            let app = find(tx, &owner)?
                .ok_or_else(|| Fault::precondition("App is no longer registered"))?;
            let placements: Vec<_> = all::<Placement>(tx, "Placement")?
                .into_iter()
                .filter(|p| p.app.id == app.id && p.revision == app.revision)
                .collect();
            let date = now();
            let ready = placements
                .iter()
                .filter(|p| {
                    p.phase == PlacementPhase::Ready
                        && p.execution.phase == Phase::Running
                        && !p.execution.draining
                        && p.date_seen <= date
                        && date.saturating_sub(p.date_seen) <= fresh
                })
                .count() as u32;
            let error = placements
                .iter()
                .find(|p| p.phase != PlacementPhase::Released && !p.error.is_empty())
                .map(|p| p.error.chars().take(512).collect())
                .unwrap_or_default();
            Ok(Snapshot {
                app,
                generation: record.generation,
                ready,
                error,
            })
        })
        .await
}

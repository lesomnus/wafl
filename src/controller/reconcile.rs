use super::*;
use crate::services::wafl::{InstanceServiceClient, ReservationServiceClient, RunnerServiceClient};
use connectrpc::{
    ErrorCode,
    client::{ClientConfig, HttpClient},
};
use futures_util::{StreamExt, stream};
use paydirt_store::{Fault, Transaction};
use std::collections::{BTreeMap, BTreeSet};

fn client(
    endpoint: &str,
    session: Option<&[u8]>,
    revision: Option<u64>,
    timeout: u64,
) -> ClientConfig {
    let mut headers = http::HeaderMap::new();
    if let Some(session) = session {
        headers.insert(
            "wafl-session",
            paydirt::format_id(session).unwrap().parse().unwrap(),
        );
    }
    if let Some(revision) = revision {
        headers.insert("wafl-revision", revision.to_string().parse().unwrap());
    }
    ClientConfig::new(endpoint.parse().unwrap())
        .with_default_headers(headers)
        .with_default_timeout(Duration::from_millis(timeout))
}
fn reference(p: &Placement) -> InstanceRef {
    InstanceRef {
        key: Some(instance_ref::Key::Id(p.instance.clone())),
        ..Default::default()
    }
}
fn patch(
    tx: &mut Transaction<'_>,
    id: &[u8],
    mut fields: Value,
) -> paydirt_store::Result<Placement> {
    fields["ref"] = json!({"id":encode(id)});
    decode(call(tx, "Placement", "Patch", fields)?)
}
fn active(p: &Placement) -> bool {
    p.phase == PlacementPhase::Pending || p.phase == PlacementPhase::Ready
}

impl Control {
    async fn observe(&self) -> BTreeMap<String, Runner> {
        stream::iter(self.config.runners.iter().cloned())
            .map(|endpoint| {
                let timeout = self.config.probe_ms;
                async move {
                    let client = RunnerServiceClient::new(
                        HttpClient::plaintext(),
                        client(&endpoint, None, None, timeout),
                    );
                    let runner = client
                        .get(RunnerGetRequest {
                            r#ref: Some(RunnerRef {
                                key: Some(runner_ref::Key::Alias("self".into())),
                                ..Default::default()
                            })
                            .into(),
                            ..Default::default()
                        })
                        .await
                        .ok()?
                        .into_owned();
                    (paydirt::has_domain(&runner.id, 8)
                        && runner
                            .capabilities
                            .iter()
                            .any(|c| c == "direct-endpoint-v1"))
                    .then_some((endpoint, runner))
                }
            })
            .buffer_unordered(8)
            .filter_map(|v| async move { v })
            .collect()
            .await
    }
    async fn plan(
        &self,
        runners: BTreeMap<String, Runner>,
    ) -> Result<Vec<Placement>, ConnectError> {
        let config = self.config.clone();
        self.db.transaction(move |tx| {
            let apps = all::<App>(tx, "App")?;
            let mut placements = all::<Placement>(tx, "Placement")?;
            placements.sort_by(|a,b| a.id.cmp(&b.id));
            let date = now();
            for app in apps {
                let mut kept = 0;
                for p in placements.iter_mut().filter(|p| p.app.id == app.id && active(p)) {
                    let owner = runners.get(&p.runner);
                    let stale = date.saturating_sub(p.date_seen) > config.fresh_ms;
                    let unavailable = owner.is_none_or(|r| r.id != p.session);
                    if p.revision != app.revision || kept >= app.replicas || stale && (unavailable || p.phase == PlacementPhase::Ready) {
                        *p = patch(tx, &p.id, json!({"phase":PlacementPhase::Retiring}))?;
                    } else {
                        kept += 1;
                        // Pending Add can take longer than a probe cycle. A live
                        // owning session keeps its durable intent allocated.
                        if p.phase == PlacementPhase::Pending && !unavailable {
                            *p = patch(tx, &p.id, json!({"dateSeen":date.to_string()}))?;
                        }
                    }
                }
                if kept >= app.replicas || placements.iter().any(|p| p.app.id == app.id && p.phase == PlacementPhase::Pending)
                    || placements.iter().filter(|p| p.phase == PlacementPhase::Pending).count() >= config.max_pending { continue; }
                // Do not turn a bad bundle into a tight crash loop or accumulate
                // unbounded executions while gateway withdrawal is unconfirmed.
                let retiring: Vec<_> = placements.iter().filter(|p| p.app.id == app.id && p.phase == PlacementPhase::Retiring).collect();
                if retiring.len() >= config.max_retiring || retiring.iter().any(|p| p.revision == app.revision && p.execution.phase == Phase::Failed) { continue; }
                let memory = app.manifest.memory_bytes;
                let mut candidates: Vec<_> = runners.iter().filter(|(endpoint,r)| {
                    let owned: Vec<_> = placements.iter().filter(|p| p.runner == **endpoint && p.session == r.id).collect();
                    let pending: Vec<_> = owned.iter().filter(|p| p.phase == PlacementPhase::Pending).collect();
                    let allocated: Vec<_> = owned.iter().filter(|p| p.phase != PlacementPhase::Released).collect();
                    let pending_memory = pending.iter().fold(0u64, |n,p| n.saturating_add(p.manifest.memory_bytes));
                    let allocated_memory = allocated.iter().fold(0u64, |n,p| n.saturating_add(p.manifest.memory_bytes));
                    // A concurrent controller may have recorded Ready since our
                    // probe. Account for durable reservations as well as usage.
                    r.accepting && r.usage.running.saturating_add(pending.len() as u64).max(allocated.len() as u64) < r.capacity.running
                        && r.usage.instances.saturating_add(pending.len() as u64).max(owned.len() as u64) < r.capacity.instances
                        && r.usage.memory_bytes.saturating_add(pending_memory).max(allocated_memory).saturating_add(memory) <= r.capacity.memory_bytes
                }).collect();
                candidates.sort_by_key(|(endpoint,r)| {
                    let existing = placements.iter().any(|p| p.runner == **endpoint && p.app.id == app.id && p.revision == app.revision && active(p));
                    let pending = placements.iter().filter(|p| p.runner == **endpoint && p.phase == PlacementPhase::Pending).count();
                    (!existing, r.usage.running.saturating_add(pending as u64), (*endpoint).clone())
                });
                let Some((endpoint,r)) = candidates.first() else { continue; };
                let p: Placement = decode(call(tx, "Placement", "Add", json!({
                    "id":encode(&paydirt::mint(11)), "tenant":{"id":encode(&app.tenant.id)},
                    "app":{"id":encode(&app.id)}, "revision":app.revision.to_string(),
                    "runner":endpoint, "session":encode(&r.id), "instance":encode(&paydirt::mint(7)),
                    "manifest":app.manifest, "phase":PlacementPhase::Pending,"dateSeen":date.to_string()
                }))?)?;
                placements.push(p);
            }
            Ok(placements.into_iter().filter(|p| p.phase != PlacementPhase::Released).collect())
        }).await
    }
    pub(super) async fn run(&self, stop: paydirt::spin::Stop) -> Result<(), String> {
        let mut tick = tokio::time::interval(Duration::from_millis(self.config.interval_ms));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut jobs = tokio::task::JoinSet::new();
        let mut working = BTreeSet::new();
        let mut rotation = 0usize;
        loop {
            tokio::select! {
                _ = stop.cancelled() => break,
                result = jobs.join_next(), if !jobs.is_empty() => {
                    match result {
                        Some(Ok(id)) => { working.remove(&id); }
                        Some(Err(e)) => return Err(format!("controller worker failed: {e}")),
                        None => {},
                    }
                },
                _ = tick.tick() => {
                    let observations = tokio::select! { _=stop.cancelled()=>break, value=self.observe()=>value };
                    let plans = self.plan(observations).await;
                    let Ok(mut plans) = plans else { eprintln!("controller reconciliation: database unavailable"); continue; };
                    if !plans.is_empty() { let n=plans.len(); plans.rotate_left(rotation % n); rotation=rotation.wrapping_add(64); }
                    for p in plans {
                        if jobs.len() >= 64 { break; }
                        if !working.insert(p.id.clone()) { continue; }
                        let controller = self.clone();
                        jobs.spawn(async move {
                            if let Err(error) = controller.process(p.clone()).await {
                                let id=p.id.clone();
                                let message=error.message.unwrap_or_else(|| "runner operation failed".into());
                                let _=controller.db.transaction(move |tx| {
                                    let p: Placement=decode(call(tx,"Placement","Get",json!({"ref":{"id":encode(&id)}}))?)?;
                                    if p.phase != PlacementPhase::Released {
                                        let mut fields=json!({"error":message});
                                        if p.phase == PlacementPhase::Pending {
                                            let failures=p.failures.saturating_add(1);
                                            let delay=1000u64.saturating_mul(1 << failures.saturating_sub(1).min(6)).min(60000);
                                            fields["failures"]=json!(failures);
                                            fields["dateRetry"]=json!(now().saturating_add(delay).to_string());
                                        }
                                        patch(tx, &id, fields)?;
                                    }
                                    Ok(())
                                }).await;
                            }
                            p.id
                        });
                    }
                }
            }
        }
        jobs.abort_all();
        while jobs.join_next().await.is_some() {}
        Ok(())
    }
    async fn record(&self, p: &Placement, execution: Instance) -> Result<(), ConnectError> {
        let p = p.clone();
        self.db.transaction(move |tx| {
            let current: Placement = decode(call(tx, "Placement", "Get", json!({"ref":{"id":encode(&p.id)}}))?)?;
            if current.phase == PlacementPhase::Released { return Ok(()); }
            if current.execution.revision > execution.revision { return Ok(()); }
            let app: App = decode(call(tx, "App", "Get", json!({"ref":{"id":encode(&p.app.id)}}))?)?;
            let mut phase = PlacementPhase::Retiring;
            if active(&current) && app.revision == p.revision && app.replicas > 0 {
                if execution.phase == Phase::Running && !execution.draining { phase = PlacementPhase::Ready; }
                else if execution.phase == Phase::Preparing || execution.phase == Phase::Starting { phase = PlacementPhase::Pending; }
            }
            // Pin the first successful bundle digest before reserving additional
            // copies of this revision on other runners.
            if phase == PlacementPhase::Ready && app.manifest.bundle.digest.is_empty() {
                let mut manifest = (*app.manifest).clone();
                manifest.bundle.get_or_insert_default().digest = execution.manifest.bundle.digest.clone();
                call(tx, "App", "Patch", json!({"ref":{"id":encode(&app.id)},"manifest":manifest}))?;
            }
            let error = execution.error.clone();
            let mut fields=json!({"phase":phase,"execution":execution,"dateSeen":now().to_string(),"error":error});
            if phase == PlacementPhase::Ready {fields["failures"]=json!(0);fields["dateRetry"]=json!("0");}
            patch(tx, &p.id, fields)?;
            Ok(())
        }).await
    }
    fn verify(&self, p: &Placement, execution: &Instance) -> Result<(), ConnectError> {
        let mut expected = (*p.manifest).clone();
        if expected.bundle.digest.is_empty() {
            expected.bundle.get_or_insert_default().digest =
                execution.manifest.bundle.digest.clone();
        }
        if execution.id != p.instance
            || execution.session != p.session
            || execution.tenant.id != p.tenant.id
            || !execution.direct
            || *execution.manifest != expected
            || p.execution.generation != 0 && execution.generation != p.execution.generation
        {
            return Err(ConnectError::failed_precondition(
                "runner execution differs from immutable placement identity/manifest",
            ));
        }
        if execution.phase == Phase::Running
            && (execution
                .endpoint
                .address
                .parse::<std::net::SocketAddrV4>()
                .is_err()
                || !matches!(execution.endpoint.protocol.as_str(), "http" | "h2c")
                || execution.manifest.bundle.digest.is_empty())
        {
            return Err(ConnectError::failed_precondition(
                "runner returned an invalid direct endpoint or digest",
            ));
        }
        Ok(())
    }
    async fn process(&self, planned: Placement) -> Result<(), ConnectError> {
        let id = planned.id.clone();
        let p: Placement = self
            .db
            .transaction(move |tx| {
                decode(call(
                    tx,
                    "Placement",
                    "Get",
                    json!({"ref":{"id":encode(&id)}}),
                )?)
            })
            .await?;
        if p.phase == PlacementPhase::Released {
            return Ok(());
        }
        if p.phase == PlacementPhase::Pending && p.date_retry > now() {
            return Ok(());
        }
        let rc = RunnerServiceClient::new(
            HttpClient::plaintext(),
            client(&p.runner, None, None, self.config.probe_ms),
        );
        let owner = rc
            .get(RunnerGetRequest {
                r#ref: Some(RunnerRef {
                    key: Some(runner_ref::Key::Alias("self".into())),
                    ..Default::default()
                })
                .into(),
                ..Default::default()
            })
            .await?
            .into_owned();
        if owner.id != p.session {
            if p.phase == PlacementPhase::Retiring && p.withdrawn {
                return self.orphan(&p, &owner).await;
            }
            return Err(ConnectError::failed_precondition(
                "runner session changed; placement will retire before replacement",
            ));
        }
        let ic = InstanceServiceClient::new(
            HttpClient::plaintext(),
            client(&p.runner, Some(&p.session), None, self.config.operation_ms),
        );
        let execution = match ic
            .get(InstanceGetRequest {
                r#ref: Some(reference(&p)).into(),
                ..Default::default()
            })
            .await
        {
            Ok(value) => value.into_owned(),
            Err(e) if e.code == ErrorCode::NotFound && p.phase == PlacementPhase::Pending => {
                let added = ic
                    .add(InstanceAddRequest {
                        id: Some(p.instance.clone()),
                        tenant: Some(TenantRef {
                            key: Some(tenant_ref::Key::Id(p.tenant.id.clone())),
                            ..Default::default()
                        })
                        .into(),
                        alias: Some(format!("p-{}", paydirt::format_id(&p.id).unwrap())),
                        manifest: p.manifest.clone(),
                        ..Default::default()
                    })
                    .await;
                match added {
                    Ok(value) => value.into_owned(),
                    Err(e) if e.code == ErrorCode::AlreadyExists => ic
                        .get(InstanceGetRequest {
                            r#ref: Some(reference(&p)).into(),
                            ..Default::default()
                        })
                        .await?
                        .into_owned(),
                    Err(e) => return Err(e),
                }
            }
            Err(e) => return Err(e),
        };
        self.verify(&p, &execution)?;
        if p.phase != PlacementPhase::Retiring {
            return self.record(&p, execution).await;
        }
        // Retiring is terminal: a late Add/result never republishes this slot.
        let mut execution = execution;
        if !execution.draining {
            execution = InstanceServiceClient::new(
                HttpClient::plaintext(),
                client(
                    &p.runner,
                    Some(&p.session),
                    Some(execution.revision),
                    self.config.probe_ms,
                ),
            )
            .drain(InstanceDrainRequest {
                r#ref: Some(reference(&p)).into(),
                ..Default::default()
            })
            .await?
            .into_owned();
            self.verify(&p, &execution)?;
        }
        self.record(&p, execution.clone()).await?;
        if !p.withdrawn {
            return Ok(());
        }
        if execution.endpoint.as_option().is_some() {
            InstanceServiceClient::new(
                HttpClient::plaintext(),
                client(
                    &p.runner,
                    Some(&p.session),
                    Some(execution.revision),
                    self.config.operation_ms,
                ),
            )
            .release(InstanceReleaseRequest {
                r#ref: Some(reference(&p)).into(),
                generation: execution.generation,
                ..Default::default()
            })
            .await?;
        }
        self.released(&p).await
    }
    async fn released(&self, p: &Placement) -> Result<(), ConnectError> {
        let id = p.id.clone();
        self.db
            .transaction(move |tx| {
                let current: Placement = decode(call(
                    tx,
                    "Placement",
                    "Get",
                    json!({"ref":{"id":encode(&id)}}),
                )?)?;
                if current.phase == PlacementPhase::Released {
                    return Ok(());
                }
                if current.phase != PlacementPhase::Retiring || !current.withdrawn {
                    return Err(Fault::precondition("withdrawal acknowledgement is missing"));
                }
                patch(
                    tx,
                    &id,
                    json!({"phase":PlacementPhase::Released,"error":""}),
                )?;
                Ok(())
            })
            .await
    }
    async fn orphan(&self, p: &Placement, owner: &Runner) -> Result<(), ConnectError> {
        let rc = ReservationServiceClient::new(
            HttpClient::plaintext(),
            client(&p.runner, Some(&owner.id), None, self.config.probe_ms),
        );
        // Without an observed address, the original Add result is uncertain.
        // Do not infer a free allocation from a different runner's empty pool.
        let address = &p.execution.endpoint.address;
        if address.is_empty() {
            return Err(ConnectError::failed_precondition(
                "uncertain Add has no observed endpoint; inspect the original runner's reservations",
            ));
        }
        let reference = ReservationRef {
            key: Some(reservation_ref::Key::Address(address.clone())),
            ..Default::default()
        };
        let reservation = match rc
            .get(ReservationGetRequest {
                r#ref: Some(reference.clone()).into(),
                ..Default::default()
            })
            .await
        {
            Ok(value) => value.into_owned(),
            Err(e) if e.code == ErrorCode::NotFound => return self.released(p).await,
            Err(e) => return Err(e),
        };
        if reservation.session != p.session
            || reservation.instance != p.instance
            || reservation.generation != p.execution.generation
        {
            return Err(ConnectError::aborted(
                "reservation now belongs to a different allocation; no release was sent",
            ));
        }
        rc.release(ReservationReleaseRequest {
            r#ref: Some(reference).into(),
            token: reservation.token,
            withdrawn: true,
            ..Default::default()
        })
        .await?;
        self.released(p).await
    }
}

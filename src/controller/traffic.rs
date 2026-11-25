use super::*;
use crate::services::wafl::{TrafficService, TrafficServiceLayer};
use connectrpc::{RequestContext, Response, ServiceRequest, ServiceResult};
use paydirt_store::{Fault, Transaction};

pub(super) fn patch(
    tx: &mut Transaction<'_>,
    id: &[u8],
    mut fields: Value,
) -> paydirt_store::Result<Traffic> {
    fields["ref"] = json!({"id":encode(id)});
    decode(call(tx, "Traffic", "Patch", fields)?)
}
fn get(tx: &mut Transaction<'_>, reference: Value) -> paydirt_store::Result<Traffic> {
    decode(call(tx, "Traffic", "Get", json!({"ref":reference}))?)
}
pub(super) fn retiring(tx: &mut Transaction<'_>, t: &Traffic) -> paydirt_store::Result<bool> {
    if t.draining || t.closed {
        return Ok(true);
    }
    let Some(p) = t.placement.as_option() else {
        return Ok(false);
    };
    let p: Placement = decode(call(
        tx,
        "Placement",
        "Get",
        json!({"ref":{"id":encode(&p.id)}}),
    )?)?;
    let a: App = decode(call(
        tx,
        "App",
        "Get",
        json!({"ref":{"id":encode(&t.app.id)}}),
    )?)?;
    Ok(p.phase != PlacementPhase::Ready || p.revision != a.revision || scaling::desired(&a) == 0)
}

#[allow(refining_impl_trait)]
impl<N: TrafficService> TrafficServiceLayer<N> for Control {
    async fn get(
        &self,
        _: &N,
        _: RequestContext,
        req: ServiceRequest<'_, TrafficGetRequest>,
    ) -> ServiceResult<Traffic> {
        let req = serde_json::to_value(req.to_owned_message()).unwrap();
        Response::ok(
            self.db
                .transaction(move |tx| decode(call(tx, "Traffic", "Get", req)?))
                .await?,
        )
    }
    async fn list(
        &self,
        _: &N,
        _: RequestContext,
        req: ServiceRequest<'_, TrafficListRequest>,
    ) -> ServiceResult<TrafficListResponse> {
        let req = serde_json::to_value(req.to_owned_message()).unwrap();
        Response::ok(
            self.db
                .transaction(move |tx| decode(call(tx, "Traffic", "List", req)?))
                .await?,
        )
    }
    async fn add(
        &self,
        _: &N,
        _: RequestContext,
        req: ServiceRequest<'_, TrafficAddRequest>,
    ) -> ServiceResult<Traffic> {
        let mut req = serde_json::to_value(req.to_owned_message()).unwrap();
        // Only these caller fields have meaning. Tenant/app/revision are resolved
        // from the placement, and observer counters always start at zero.
        let allowed = ["id", "placement", "session"];
        if req
            .as_object()
            .unwrap()
            .keys()
            .any(|k| !allowed.contains(&k.as_str()))
        {
            return Err(ConnectError::invalid_argument(
                "Traffic.Add accepts id, placement and session only",
            ));
        }
        let id = req["id"]
            .as_str()
            .and_then(|s| STANDARD.decode(s).ok())
            .filter(|id| paydirt::has_domain(id, 13))
            .ok_or_else(|| {
                ConnectError::invalid_argument("a caller-chosen domain-13 id is required")
            })?;
        let session = req["session"]
            .as_str()
            .filter(|s| {
                !s.is_empty()
                    && s.len() <= 128
                    && s.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
            })
            .ok_or_else(|| ConnectError::invalid_argument("invalid observer session"))?
            .to_owned();
        Response::ok(
            self.db
                .transaction(move |tx| {
                    let p: Placement = decode(call(
                        tx,
                        "Placement",
                        "Get",
                        json!({"ref":req["placement"]}),
                    )?)?;
                    let a: App = decode(call(
                        tx,
                        "App",
                        "Get",
                        json!({"ref":{"id":encode(&p.app.id)}}),
                    )?)?;
                    let page = call(
                        tx,
                        "Traffic",
                        "List",
                        json!({"filters":[{"ref":{"id":encode(&id)}}]}),
                    )?;
                    if let Some(row) = page["items"].as_array().and_then(|v| v.first()) {
                        let t: Traffic = decode(row.clone())?;
                        if t.session != session || t.placement.id != p.id {
                            return Err(Fault::precondition("observer identity differs"));
                        }
                        return if retiring(tx, &t)? {
                            patch(tx, &t.id, json!({"draining":true}))
                        } else {
                            Ok(t)
                        };
                    }
                    if !p.managed
                        || p.phase != PlacementPhase::Ready
                        || p.revision != a.revision
                        || scaling::desired(&a) == 0
                    {
                        return Err(Fault::precondition(
                            "placement does not accept new observers",
                        ));
                    }
                    req["tenant"] = json!({"id":encode(&p.tenant.id)});
                    req["app"] = json!({"id":encode(&a.id)});
                    req["revision"] = json!(a.revision.to_string());
                    req["dateSeen"] = json!(now().to_string());
                    decode(call(tx, "Traffic", "Add", req)?)
                })
                .await?,
        )
    }
    async fn report(
        &self,
        _: &N,
        _: RequestContext,
        req: ServiceRequest<'_, TrafficReportRequest>,
    ) -> ServiceResult<Traffic> {
        let req = req.to_owned_message();
        Response::ok(self.db.transaction(move|tx| {
            let t=get(tx,serde_json::to_value(req.r#ref).unwrap())?;
            if t.activation {return Err(Fault::precondition("activation observations are internal"));}
            let draining=retiring(tx,&t)?;
            // Retries/out-of-order delivery cannot refresh or reopen a gate.
            if req.sequence<=t.sequence || t.closed && t.active==0 {
                return if draining && !t.draining {patch(tx,&t.id,json!({"draining":true}))} else {Ok(t)};
            }
            if req.started<t.started || req.busy_ms<t.busy_ms || u64::from(req.active)>req.started
                || t.closed && (!req.closed || req.active>t.active || req.started!=t.started) {
                return Err(Fault::invalid("observer counters regressed or a closed gate reopened"));
            }
            patch(tx,&t.id,json!({"sequence":req.sequence.to_string(),"active":req.active,
                "started":req.started.to_string(),"busyMs":req.busy_ms.to_string(),"closed":req.closed,
                "draining":draining,"dateSeen":now().to_string()}))
        }).await?)
    }
    async fn close(
        &self,
        _: &N,
        _: RequestContext,
        req: ServiceRequest<'_, TrafficCloseRequest>,
    ) -> ServiceResult<Traffic> {
        let req = req.to_owned_message();
        if !req.withdrawn {
            return Err(ConnectError::invalid_argument(
                "withdrawn=true requires the observer and every owned connection to have stopped",
            ));
        }
        Response::ok(self.db.transaction(move|tx| {
            let t=get(tx,serde_json::to_value(req.r#ref).unwrap())?;
            patch(tx,&t.id,json!({"closed":true,"draining":true,"active":0,"dateSeen":now().to_string()}))
        }).await?)
    }
}

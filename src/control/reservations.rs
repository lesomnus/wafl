use super::*;
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReservationCursor {
    id: Vec<u8>,
    reverse: bool,
    scope: String,
    filters: String,
}

#[allow(refining_impl_trait)]
impl<N: ReservationService> ReservationServiceLayer<N> for Instances {
    async fn get(
        &self,
        _: &N,
        ctx: RequestContext,
        req: ServiceRequest<'_, ReservationGetRequest>,
    ) -> ServiceResult<Reservation> {
        self.0.check_session(condition(&ctx)?.session)?;
        let req = req.to_owned_message();
        let mut value = self.0.ports()?.get((*req.r#ref).clone()).await?;
        if let Some(select) = req.select.as_option()
            && !select.all.unwrap_or(false)
        {
            if !select.address.unwrap_or(false) {
                value.address.clear();
            }
            if !select.token.unwrap_or(false) {
                value.token.clear();
            }
            if !select.session.unwrap_or(false) {
                value.session.clear();
            }
            if !select.instance.unwrap_or(false) {
                value.instance.clear();
            }
            if !select.generation.unwrap_or(false) {
                value.generation = 0;
            }
            if !select.status.unwrap_or(false) {
                value.status = Default::default();
            }
            if !select.error.unwrap_or(false) {
                value.error.clear();
            }
        }
        Response::ok(value)
    }

    async fn list(
        &self,
        _: &N,
        ctx: RequestContext,
        req: ServiceRequest<'_, ReservationListRequest>,
    ) -> ServiceResult<ReservationListResponse> {
        self.0.check_session(condition(&ctx)?.session)?;
        let mut req = req.to_owned_message();
        if req.filters.len() > 64 {
            return Err(ConnectError::invalid_argument(
                "at most 64 filters are allowed",
            ));
        }
        let pool = self.0.ports()?;
        for filter in &mut req.filters {
            match &mut filter.r#ref.get_or_insert_default().key {
                Some(reservation_ref::Key::Id(id))
                    if paydirt::has_domain(id, RESERVATION_DOMAIN) => {}
                Some(reservation_ref::Key::Address(address)) => {
                    let parsed: std::net::SocketAddr = address.parse().map_err(|_| {
                        ConnectError::invalid_argument("invalid reservation address")
                    })?;
                    if parsed.ip() != std::net::IpAddr::V4(pool.config.advertise)
                        || !(pool.config.first..=pool.config.last).contains(&parsed.port())
                    {
                        return Err(ConnectError::invalid_argument(
                            "address is outside this runner's direct pool",
                        ));
                    }
                    *address = parsed.to_string();
                }
                _ => return Err(ConnectError::invalid_argument("invalid reservation filter")),
            }
        }
        let filter_hash = format!(
            "{:x}",
            Sha256::digest(
                serde_json::to_vec(&req.filters)
                    .map_err(|_| ConnectError::internal("filter encoding failed"))?
            )
        );
        let scope = pool.scope();
        let mut items = pool.list().await?;
        items.retain(|item| {
            req.filters.is_empty()
                || req.filters.iter().any(|filter| match &filter.r#ref.key {
                    Some(reservation_ref::Key::Id(id)) => *id == item.id,
                    Some(reservation_ref::Key::Address(address)) => *address == item.address,
                    None => false,
                })
        });
        items.sort_by(|a, b| a.id.cmp(&b.id));
        if req.reverse {
            items.reverse();
        }
        if !req.after.is_empty() {
            let invalid = || {
                ConnectError::invalid_argument(
                    "invalid reservation cursor for this pool/order/filter",
                )
            };
            if req.after.len() > 512 {
                return Err(invalid());
            }
            let cursor: ReservationCursor =
                serde_json::from_slice(&URL_SAFE_NO_PAD.decode(&req.after).map_err(|_| invalid())?)
                    .map_err(|_| invalid())?;
            if cursor.reverse != req.reverse
                || cursor.scope != scope
                || cursor.filters != filter_hash
                || !paydirt::has_domain(&cursor.id, RESERVATION_DOMAIN)
            {
                return Err(invalid());
            }
            items.retain(|item| {
                if req.reverse {
                    item.id < cursor.id
                } else {
                    item.id > cursor.id
                }
            });
        }
        let size = if req.size <= 0 {
            100
        } else {
            req.size.min(1000) as usize
        };
        let next = if items.len() > size {
            URL_SAFE_NO_PAD.encode(
                serde_json::to_vec(&ReservationCursor {
                    id: items[size - 1].id.clone(),
                    reverse: req.reverse,
                    scope,
                    filters: filter_hash,
                })
                .map_err(|_| ConnectError::internal("cursor encoding failed"))?,
            )
        } else {
            String::new()
        };
        items.truncate(size);
        Response::ok(ReservationListResponse {
            items,
            next,
            ..Default::default()
        })
    }

    async fn release(
        &self,
        _: &N,
        ctx: RequestContext,
        req: ServiceRequest<'_, ReservationReleaseRequest>,
    ) -> ServiceResult<ReservationReleaseResponse> {
        let condition = condition(&ctx)?;
        if condition.revision.is_some() {
            return Err(ConnectError::invalid_argument(
                "reservation Release uses token, not wafl-revision",
            ));
        }
        self.0.check_session(condition.session)?;
        let req = req.to_owned_message();
        let released = self
            .0
            .ports()?
            .reclaim((*req.r#ref).clone(), req.token, req.withdrawn)
            .await?;
        Response::ok(ReservationReleaseResponse {
            released,
            ..Default::default()
        })
    }
}

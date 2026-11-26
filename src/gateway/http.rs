use super::state::{Failure, Key, State, Target};
use async_trait::async_trait;
use pingora::{
    Error, ErrorType, Result,
    http::{RequestHeader, ResponseHeader},
    prelude::{HttpPeer, ProxyHttp, Session},
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

pub(super) struct Http {
    pub state: Arc<State>,
    pub health: bool,
}
#[derive(Default)]
pub(super) struct Context {
    target: Option<Target>,
    deadline: Option<Instant>,
}

pub(super) fn grpc_timeout(value: &str) -> Option<Duration> {
    if !(2..=9).contains(&value.len()) {
        return None;
    }
    let (digits, unit) = value.split_at(value.len() - 1);
    if !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let n = digits.parse::<u64>().ok()?;
    let nanos = match unit {
        "H" => 3_600_000_000_000,
        "M" => 60_000_000_000,
        "S" => 1_000_000_000,
        "m" => 1_000_000,
        "u" => 1_000,
        "n" => 1,
        _ => return None,
    };
    Some(Duration::from_nanos(n.saturating_mul(nanos)))
}
async fn reject(session: &mut Session, Failure(status, message): Failure) -> Result<bool> {
    let grpc = session
        .req_header()
        .headers
        .get("content-type")
        .is_some_and(|v| v.as_bytes().starts_with(b"application/grpc"));
    let mut response = ResponseHeader::build(if grpc { 200 } else { status }, None)?;
    response.insert_header("content-length", "0")?;
    if grpc {
        response.insert_header("content-type", "application/grpc")?;
        response.insert_header(
            "grpc-status",
            match status {
                504 => "4",
                429 => "8",
                400 => "3",
                _ => "14",
            },
        )?;
        response.insert_header("grpc-message", message)?;
    }
    session
        .write_response_header(Box::new(response), true)
        .await?;
    Ok(true)
}
#[async_trait]
impl ProxyHttp for Http {
    type CTX = Context;
    fn new_ctx(&self) -> Context {
        Context::default()
    }
    async fn request_filter(&self, session: &mut Session, ctx: &mut Context) -> Result<bool> {
        if self.health {
            return reject(
                session,
                Failure(
                    if self.state.healthy() { 200 } else { 503 },
                    "proxy readiness",
                ),
            )
            .await;
        }
        let host = session
            .req_header()
            .uri
            .authority()
            .map(|v| v.as_str())
            .or_else(|| {
                session
                    .req_header()
                    .headers
                    .get("host")
                    .and_then(|v| v.to_str().ok())
            })
            .unwrap_or("");
        let host = host
            .parse::<http::uri::Authority>()
            .map(|a| a.host().to_ascii_lowercase());
        let Ok(host) = host else {
            return reject(session, Failure(400, "invalid request authority")).await;
        };
        if let Some(value) = session.req_header().headers.get("grpc-timeout") {
            let Some(timeout) = value.to_str().ok().and_then(grpc_timeout) else {
                return reject(session, Failure(400, "invalid grpc-timeout")).await;
            };
            ctx.deadline = Instant::now().checked_add(timeout);
        }
        let selection = self.state.target(Key::Host(host), ctx.deadline);
        let target = if let Some(closed) = session.downstream_session.watch_h2_stream_close() {
            tokio::select! {result=selection=>result,_=closed=>return Ok(true)}
        } else {
            selection.await
        };
        match target {
            Ok(target) => {
                ctx.target = Some(target);
                session.set_keepalive(Some(60));
                Ok(false)
            }
            Err(error) => reject(session, error).await,
        }
    }
    async fn upstream_peer(
        &self,
        _session: &mut Session,
        ctx: &mut Context,
    ) -> Result<Box<HttpPeer>> {
        let target = ctx
            .target
            .as_ref()
            .ok_or_else(|| Error::explain(ErrorType::HTTPStatus(503), "no admitted target"))?;
        let mut peer = HttpPeer::new(&target.endpoint.address, false, String::new());
        peer.group_key = target.group_key;
        peer.options.connection_timeout = Some(Duration::from_secs(5));
        peer.options.total_connection_timeout = Some(Duration::from_secs(5));
        peer.options.idle_timeout = Some(Duration::from_secs(60));
        if target.protocol == "h2c" {
            peer.options.set_http_version(2, 2);
        }
        Ok(Box::new(peer))
    }
    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        request: &mut RequestHeader,
        ctx: &mut Context,
    ) -> Result<()> {
        request.remove_header("wafl-instance");
        if let Some(deadline) = ctx.deadline {
            let millis = deadline
                .saturating_duration_since(Instant::now())
                .as_millis()
                .max(1);
            // Round up to seconds for deadlines beyond the eight-digit wire bound.
            let value = if millis <= 99_999_999 {
                format!("{millis}m")
            } else {
                format!("{}S", millis.div_ceil(1000).min(99_999_999))
            };
            request.insert_header("grpc-timeout", value)?;
        }
        Ok(())
    }
    async fn logging(&self, _session: &mut Session, _error: Option<&Error>, ctx: &mut Context) {
        ctx.target.take();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn grpc_deadline_contract() {
        assert_eq!(grpc_timeout("10m"), Some(Duration::from_millis(10)));
        assert_eq!(grpc_timeout("1S"), Some(Duration::from_secs(1)));
        assert!(grpc_timeout("99999999H").is_some());
        for v in ["", "10", "1x", "123456789S", "-1S"] {
            assert_eq!(grpc_timeout(v), None);
        }
    }
}

use crate::{
    config::Protocol,
    runner::{Lease, Runner},
};
use bytes::Bytes;
use http::{HeaderMap, Request, Response, StatusCode, Version, header};
use http_body::{Body, Frame, SizeHint};
use http_body_util::{BodyExt, Full, combinators::UnsyncBoxBody};
use hyper::body::Incoming;
use hyper_util::rt::{TokioExecutor, TokioIo};
use paydirt::spin::Stop;
use std::{
    convert::Infallible,
    error::Error,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
    time::Duration,
};
use tokio::{
    net::{TcpListener, TcpStream},
    task::{AbortHandle, JoinSet},
};

type BoxError = Box<dyn Error + Send + Sync>;
pub type ProxyBody = UnsyncBoxBody<Bytes, BoxError>;
/// Set by the gateway; consumed by the runner before forwarding to the guest.
pub const INSTANCE_HEADER: &str = "wafl-instance";

struct ResponseBody {
    inner: Incoming,
    lease: Arc<Mutex<Option<Lease>>>,
    expiry: Option<AbortHandle>,
    connection: AbortHandle,
    deadline: Option<Pin<Box<tokio::time::Sleep>>>,
    expired: bool,
}
impl Body for ResponseBody {
    type Data = Bytes;
    type Error = BoxError;
    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, BoxError>>> {
        if self.expired {
            return Poll::Ready(None);
        }
        if self
            .deadline
            .as_mut()
            .is_some_and(|timer| timer.as_mut().poll(cx).is_ready())
        {
            self.expired = true;
            self.connection.abort();
            self.lease.lock().unwrap().take();
            let mut trailers = HeaderMap::new();
            trailers.insert("grpc-status", http::HeaderValue::from_static("4"));
            trailers.insert(
                "grpc-message",
                http::HeaderValue::from_static("runner request deadline exceeded"),
            );
            return Poll::Ready(Some(Ok(Frame::trailers(trailers))));
        }
        match Pin::new(&mut self.inner).poll_frame(cx) {
            Poll::Ready(None) => {
                self.lease.lock().unwrap().take();
                Poll::Ready(None)
            }
            Poll::Ready(Some(Err(e))) => {
                self.lease.lock().unwrap().take();
                Poll::Ready(Some(Err(Box::new(e))))
            }
            Poll::Ready(Some(Ok(mut frame))) => {
                if let Some(trailers) = frame.trailers_mut() {
                    trailers.remove("wafl-error");
                }
                if self.inner.is_end_stream() {
                    self.lease.lock().unwrap().take();
                }
                Poll::Ready(Some(Ok(frame)))
            }
            Poll::Pending => Poll::Pending,
        }
    }
    fn is_end_stream(&self) -> bool {
        self.expired || self.inner.is_end_stream()
    }
    fn size_hint(&self) -> SizeHint {
        if self.deadline.is_some() {
            SizeHint::default()
        } else {
            self.inner.size_hint()
        }
    }
}
impl Drop for ResponseBody {
    fn drop(&mut self) {
        self.connection.abort();
        if let Some(expiry) = &self.expiry {
            expiry.abort();
        }
        self.lease.lock().unwrap().take();
    }
}

// Abort the upstream driver on every failure path, including a cancelled request
// before response headers arrive. Ownership transfers to ResponseBody on success.
struct Driver(Option<AbortHandle>);
impl Drop for Driver {
    fn drop(&mut self) {
        if let Some(task) = &self.0 {
            task.abort();
        }
    }
}

fn reply(status: StatusCode, text: &'static str) -> Response<ProxyBody> {
    Response::builder()
        .status(status)
        .body(
            Full::new(Bytes::from_static(text.as_bytes()))
                .map_err(|e: Infallible| match e {})
                .boxed_unsync(),
        )
        .unwrap()
}

fn reject(status: StatusCode, reason: &'static str, text: &'static str) -> Response<ProxyBody> {
    let mut response = reply(status, text);
    response
        .headers_mut()
        .insert("wafl-error", http::HeaderValue::from_static(reason));
    response
}

pub async fn forward(
    runner: Runner,
    request: Request<Incoming>,
) -> Result<Response<ProxyBody>, Infallible> {
    if runner.direct() {
        return Ok(reject(
            StatusCode::NOT_FOUND,
            "direct_endpoint",
            "connect to Instance.endpoint\n",
        ));
    }
    let grpc = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v == "application/grpc" || v.starts_with("application/grpc+"));
    let deadline = if grpc {
        match grpc_deadline(request.headers()) {
            Ok(deadline) => deadline,
            Err(()) => {
                return Ok(reject(
                    StatusCode::BAD_REQUEST,
                    "invalid_request",
                    "invalid grpc-timeout\n",
                ));
            }
        }
    } else {
        None
    };
    if let Some(deadline) = deadline {
        tokio::select! {
            biased;
            _ = tokio::time::sleep_until(deadline) => {
                let mut response = reply(StatusCode::OK, "");
                response.headers_mut().insert(header::CONTENT_TYPE, http::HeaderValue::from_static("application/grpc"));
                response.headers_mut().insert("grpc-status", http::HeaderValue::from_static("4"));
                response.headers_mut().insert("grpc-message", http::HeaderValue::from_static("runner request deadline exceeded"));
                response.headers_mut().insert("wafl-error", http::HeaderValue::from_static("deadline"));
                Ok(response)
            },
            result = forward_inner(runner, request, Some(deadline)) => result,
        }
    } else {
        forward_inner(runner, request, None).await
    }
}

fn grpc_deadline(headers: &HeaderMap) -> Result<Option<tokio::time::Instant>, ()> {
    let mut values = headers.get_all("grpc-timeout").iter();
    let Some(value) = values.next() else {
        return Ok(None);
    };
    if values.next().is_some() {
        return Err(());
    }
    let value = value.to_str().map_err(|_| ())?;
    if !value.is_ascii() || !(2..=9).contains(&value.len()) {
        return Err(());
    }
    let (count, unit) = value.split_at(value.len() - 1);
    if !count.bytes().all(|b| b.is_ascii_digit()) {
        return Err(());
    }
    let count = count.parse::<u64>().map_err(|_| ())?;
    let duration = match unit {
        "H" => Duration::from_secs(count * 3600),
        "M" => Duration::from_secs(count * 60),
        "S" => Duration::from_secs(count),
        "m" => Duration::from_millis(count),
        "u" => Duration::from_micros(count),
        "n" => Duration::from_nanos(count),
        _ => return Err(()),
    };
    // Zero is treated as immediately expired, never as an infinite deadline.
    tokio::time::Instant::now()
        .checked_add(duration)
        .map(Some)
        .ok_or(())
}
fn remaining(request: &mut Request<Incoming>, deadline: Option<tokio::time::Instant>) {
    let Some(deadline) = deadline else { return };
    let nanos = deadline
        .saturating_duration_since(tokio::time::Instant::now())
        .as_nanos()
        .max(1);
    for (unit, scale) in [
        ("n", 1),
        ("u", 1000),
        ("m", 1_000_000),
        ("S", 1_000_000_000),
        ("M", 60_000_000_000),
        ("H", 3_600_000_000_000),
    ] {
        let count = nanos.div_ceil(scale);
        if count <= 99_999_999 {
            request
                .headers_mut()
                .insert("grpc-timeout", format!("{count}{unit}").parse().unwrap());
            return;
        }
    }
}

async fn forward_inner(
    runner: Runner,
    mut request: Request<Incoming>,
    deadline: Option<tokio::time::Instant>,
) -> Result<Response<ProxyBody>, Infallible> {
    let authority = request.uri().authority().map(|a| a.as_str()).or_else(|| {
        request
            .headers()
            .get(header::HOST)
            .and_then(|h| h.to_str().ok())
    });
    let authority = authority
        .filter(|a| !a.contains('@'))
        .and_then(|a| a.parse::<http::uri::Authority>().ok())
        .map(|a| a.to_string());
    if authority.is_none() {
        return Ok(reject(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "invalid request authority\n",
        ));
    }
    let mut values = request.headers().get_all(INSTANCE_HEADER).iter();
    let id = values
        .next()
        .and_then(|value| value.to_str().ok())
        .and_then(paydirt::parse_id)
        .filter(|id| paydirt::has_domain(id, crate::config::INSTANCE_DOMAIN))
        .filter(|_| values.next().is_none());
    let Some(id) = id else {
        return Ok(reject(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "wafl-instance must name one Instance UUID\n",
        ));
    };
    let condition = match crate::control::condition(&connectrpc::RequestContext::new(
        request.headers().clone(),
    )) {
        Ok(condition) if condition.revision.is_none() => condition,
        _ => {
            return Ok(reject(
                StatusCode::BAD_REQUEST,
                "invalid_request",
                "invalid routing session\n",
            ));
        }
    };
    if runner.check_session(condition.session).is_err() {
        return Ok(reject(
            StatusCode::SERVICE_UNAVAILABLE,
            "session_mismatch",
            "runner session changed\n",
        ));
    }
    let Some(app) = runner.instance(&id) else {
        return Ok(if runner.snapshot(&id).is_some() {
            reject(
                StatusCode::SERVICE_UNAVAILABLE,
                "preparing",
                "instance is being added\n",
            )
        } else {
            reject(StatusCode::NOT_FOUND, "not_found", "unknown instance\n")
        });
    };
    request.headers_mut().remove(INSTANCE_HEADER);
    request.headers_mut().remove("wafl-session");
    request.headers_mut().remove("wafl-revision");
    if request.method() == http::Method::CONNECT || request.headers().contains_key(header::UPGRADE)
    {
        return Ok(reject(
            StatusCode::NOT_IMPLEMENTED,
            "unsupported",
            "protocol upgrades are not supported\n",
        ));
    }
    let lease = match runner.acquire(&app).await {
        Ok(lease) => lease,
        Err(error) => {
            use connectrpc::ErrorCode;
            let (status, reason) = match error.code {
                ErrorCode::ResourceExhausted => (StatusCode::TOO_MANY_REQUESTS, "capacity"),
                ErrorCode::DeadlineExceeded => (StatusCode::GATEWAY_TIMEOUT, "startup_timeout"),
                ErrorCode::FailedPrecondition => (StatusCode::SERVICE_UNAVAILABLE, "draining"),
                ErrorCode::NotFound => (StatusCode::NOT_FOUND, "not_found"),
                _ => (StatusCode::SERVICE_UNAVAILABLE, "unavailable"),
            };
            return Ok(reject(
                status,
                reason,
                "instance rejected request before forwarding\n",
            ));
        }
    };
    strip_hop_headers(request.headers_mut());
    let result: Result<_, BoxError> = async {
        let stream =
            tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(lease.address))
                .await??;
        stream.set_nodelay(true)?;
        let (response, mut driver) = match app.spec.protocol {
            Protocol::Http => {
                request
                    .headers_mut()
                    .insert(header::HOST, authority.as_deref().unwrap().parse()?);
                *request.version_mut() = Version::HTTP_11;
                let path = request
                    .uri()
                    .path_and_query()
                    .map(|p| p.as_str())
                    .unwrap_or("/")
                    .parse()?;
                *request.uri_mut() = path;
                let (mut sender, connection) =
                    hyper::client::conn::http1::handshake(TokioIo::new(stream)).await?;
                let task = tokio::spawn(async move {
                    let _ = connection.await;
                });
                let driver = Driver(Some(task.abort_handle()));
                sender.ready().await?;
                remaining(&mut request, deadline);
                (sender.send_request(request).await?, driver)
            }
            Protocol::H2c => {
                *request.version_mut() = Version::HTTP_2;
                let mut parts = request.uri().clone().into_parts();
                parts.scheme = Some(http::uri::Scheme::HTTP);
                parts.authority = Some(authority.as_deref().unwrap().parse()?);
                *request.uri_mut() = http::Uri::from_parts(parts)?;
                request.headers_mut().remove(header::HOST);
                let (mut sender, connection) = hyper::client::conn::http2::handshake(
                    TokioExecutor::new(),
                    TokioIo::new(stream),
                )
                .await?;
                let task = tokio::spawn(async move {
                    let _ = connection.await;
                });
                let driver = Driver(Some(task.abort_handle()));
                sender.ready().await?;
                remaining(&mut request, deadline);
                (sender.send_request(request).await?, driver)
            }
        };
        let (mut parts, body) = response.into_parts();
        strip_hop_headers(&mut parts.headers);
        parts.headers.remove("wafl-error");
        if deadline.is_some() {
            parts.headers.remove(header::CONTENT_LENGTH);
        }
        let connection = driver.0.take().unwrap();
        let lease = Arc::new(Mutex::new(Some(lease)));
        let expiry = deadline.map(|deadline| {
            let connection = connection.clone();
            let lease = lease.clone();
            // ResponseBody may not be polled while downstream flow control is
            // blocked. The deadline must still release admission and upstream.
            tokio::spawn(async move {
                tokio::time::sleep_until(deadline).await;
                connection.abort();
                lease.lock().unwrap().take();
            })
            .abort_handle()
        });
        Ok(Response::from_parts(
            parts,
            ResponseBody {
                inner: body,
                lease,
                expiry,
                connection,
                deadline: deadline.map(|deadline| Box::pin(tokio::time::sleep_until(deadline))),
                expired: false,
            }
            .boxed_unsync(),
        ))
    }
    .await;
    // A request body may already have reached the application. Never replay it.
    Ok(result.unwrap_or_else(|_| {
        reject(
            StatusCode::BAD_GATEWAY,
            "upstream",
            "app connection failed; delivery may have started\n",
        )
    }))
}

fn strip_hop_headers(headers: &mut HeaderMap) {
    let named: Vec<_> = headers
        .get_all(header::CONNECTION)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(','))
        .filter_map(|v| http::HeaderName::from_bytes(v.trim().as_bytes()).ok())
        .collect();
    for name in named {
        headers.remove(name);
    }
    for name in [
        "connection",
        "keep-alive",
        "proxy-authenticate",
        "proxy-authorization",
        "proxy-connection",
        "transfer-encoding",
        "upgrade",
    ] {
        headers.remove(name);
    }
    // HTTP/2 permits only TE: trailers; keep it for gRPC.
    if headers.get(header::TE).is_some_and(|v| v != "trailers") {
        headers.remove(header::TE);
    }
}

pub async fn serve(listener: TcpListener, runner: Runner, stop: Stop) -> anyhow::Result<()> {
    let mut connections = JoinSet::new();
    loop {
        tokio::select! {
            biased;
            _ = stop.cancelled() => break,
            Some(_) = connections.join_next() => {},
            socket = listener.accept() => {
                let (socket, _) = socket?;
                let timeout = runner.drain_timeout();
                let runner = runner.clone();
                let stop = stop.clone();
                connections.spawn(async move {
                    let service = hyper::service::service_fn(move |request| forward(runner.clone(), request));
                    let mut builder = hyper_util::server::conn::auto::Builder::new(TokioExecutor::new());
                    builder.http1().header_read_timeout(Duration::from_secs(10)).timer(hyper_util::rt::TokioTimer::new());
                    let connection = builder.serve_connection(TokioIo::new(socket), service);
                    tokio::pin!(connection);
                    tokio::select! {
                        _ = &mut connection => {},
                        _ = stop.cancelled() => {
                            connection.as_mut().graceful_shutdown();
                            let _ = tokio::time::timeout(timeout, connection).await;
                        }
                    }
                });
            }
        }
    }
    while connections.join_next().await.is_some() {}
    Ok(())
}

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
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::{
    net::{TcpListener, TcpStream},
    task::{AbortHandle, JoinSet},
};

type BoxError = Box<dyn Error + Send + Sync>;
pub type ProxyBody = UnsyncBoxBody<Bytes, BoxError>;

struct ResponseBody {
    inner: Incoming,
    lease: Option<Lease>,
    connection: AbortHandle,
}
impl Body for ResponseBody {
    type Data = Bytes;
    type Error = BoxError;
    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, BoxError>>> {
        match Pin::new(&mut self.inner).poll_frame(cx) {
            Poll::Ready(None) => {
                self.lease.take();
                Poll::Ready(None)
            }
            Poll::Ready(Some(Err(e))) => {
                self.lease.take();
                Poll::Ready(Some(Err(Box::new(e))))
            }
            Poll::Ready(Some(Ok(frame))) => {
                if self.inner.is_end_stream() {
                    self.lease.take();
                }
                Poll::Ready(Some(Ok(frame)))
            }
            Poll::Pending => Poll::Pending,
        }
    }
    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }
    fn size_hint(&self) -> SizeHint {
        self.inner.size_hint()
    }
}
impl Drop for ResponseBody {
    fn drop(&mut self) {
        self.connection.abort();
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

pub async fn forward(
    runner: Runner,
    mut request: Request<Incoming>,
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
    let domain = authority
        .as_deref()
        .and_then(|a| a.parse::<http::uri::Authority>().ok())
        .map(|a| a.host().trim_end_matches('.').to_ascii_lowercase());
    let Some(app) = domain.as_deref().and_then(|name| runner.domain(name)) else {
        return Ok(reply(StatusCode::NOT_FOUND, "unknown app\n"));
    };
    if request.method() == http::Method::CONNECT || request.headers().contains_key(header::UPGRADE)
    {
        return Ok(reply(
            StatusCode::NOT_IMPLEMENTED,
            "protocol upgrades are not supported\n",
        ));
    }
    let lease = match runner.acquire(&app).await {
        Ok(lease) => lease,
        Err(_) => return Ok(reply(StatusCode::SERVICE_UNAVAILABLE, "app unavailable\n")),
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
                (sender.send_request(request).await?, driver)
            }
        };
        let (mut parts, body) = response.into_parts();
        strip_hop_headers(&mut parts.headers);
        Ok(Response::from_parts(
            parts,
            ResponseBody {
                inner: body,
                lease: Some(lease),
                connection: driver.0.take().unwrap(),
            }
            .boxed_unsync(),
        ))
    }
    .await;
    // A request body may already have reached the application. Never replay it.
    Ok(result.unwrap_or_else(|_| reply(StatusCode::BAD_GATEWAY, "app connection failed\n")))
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
                            let _ = tokio::time::timeout(Duration::from_secs(5), connection).await;
                        }
                    }
                });
            }
        }
    }
    while connections.join_next().await.is_some() {}
    Ok(())
}

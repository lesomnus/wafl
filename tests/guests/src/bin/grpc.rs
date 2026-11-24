//! Test-only HTTP/2 gRPC echo server. A small polling executor lets the same
//! standard socket fixture run natively and on WASIp2 without a Wafl SDK.
//! This deliberately simple executor is not an application framework.
use bytes::Bytes;
use futures_util::{StreamExt, stream::FuturesUnordered};
use std::{
    future::Future,
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    pin::Pin,
    task::{Context, Poll, Waker},
    time::Duration,
};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

struct Io(TcpStream);
impl AsyncRead for Io {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match self.0.read(buf.initialize_unfilled()) {
            Ok(n) => {
                buf.advance(n);
                Poll::Ready(Ok(()))
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Poll::Pending,
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}
impl AsyncWrite for Io {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
        bytes: &[u8],
    ) -> Poll<io::Result<usize>> {
        match self.0.write(bytes) {
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Poll::Pending,
            result => Poll::Ready(result),
        }
    }
    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(self.0.shutdown(std::net::Shutdown::Write))
    }
}

fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = std::pin::pin!(future);
    let mut cx = Context::from_waker(Waker::noop());
    loop {
        if let Poll::Ready(value) = future.as_mut().poll(&mut cx) {
            return value;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

async fn echo(
    request: http::Request<h2::RecvStream>,
    mut response: h2::server::SendResponse<Bytes>,
) -> Result<(), h2::Error> {
    let headers = http::Response::builder()
        .header("content-type", "application/grpc")
        .header(
            "echo-timeout",
            request
                .headers()
                .get("grpc-timeout")
                .cloned()
                .unwrap_or_else(|| "none".parse().unwrap()),
        )
        .body(())
        .unwrap();
    let mut output = response.send_response(headers, false)?;
    let mut input = request.into_body();
    while let Some(bytes) = input.data().await {
        let bytes = bytes?;
        input.flow_control().release_capacity(bytes.len())?;
        output.send_data(bytes, false)?;
    }
    let mut trailers = http::HeaderMap::new();
    trailers.insert("grpc-status", "0".parse().unwrap());
    output.send_trailers(trailers)
}

async fn serve(stream: TcpStream) -> Result<(), h2::Error> {
    let mut connection = h2::server::handshake(Io(stream)).await?;
    let mut requests = FuturesUnordered::new();
    std::future::poll_fn(|cx| {
        loop {
            match connection.poll_accept(cx) {
                Poll::Ready(Some(Ok((request, response)))) => {
                    requests.push(Box::pin(echo(request, response)))
                }
                Poll::Ready(Some(Err(e))) => return Poll::Ready(Err(e)),
                Poll::Ready(None) => return Poll::Ready(Ok(())),
                Poll::Pending => break,
            }
        }
        while let Poll::Ready(Some(_)) = requests.poll_next_unpin(cx) {}
        Poll::Pending
    })
    .await
}

fn main() -> io::Result<()> {
    if std::env::var("MODE").as_deref() == Ok("delay") {
        std::thread::sleep(Duration::from_millis(200));
    }
    let listener = TcpListener::bind(("0.0.0.0", 8080))?;
    for stream in listener.incoming() {
        let stream = stream?;
        stream.set_nonblocking(true)?;
        let _ = block_on(serve(stream));
    }
    Ok(())
}

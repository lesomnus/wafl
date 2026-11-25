//! Benchmark-only HTTP/1.1 server. Poll idle connections without blocking accept.
//! The same binary serves both close and keep-alive cases; no timer/busy loop.
use std::{env, hint::black_box};
use wasip2::{
    io::{
        poll::{Pollable, poll},
        streams::{InputStream, OutputStream},
    },
    sockets::{
        instance_network::instance_network,
        network::{ErrorCode, IpAddressFamily, IpSocketAddress, Ipv4SocketAddress},
        tcp::TcpSocket,
        tcp_create_socket::create_tcp_socket,
    },
};

fn number(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .map(|value| value.parse().unwrap())
        .unwrap_or(default)
}

struct Connection {
    // WASI child resources must be dropped before their parent resources.
    readable: Pollable,
    input: InputStream,
    output: OutputStream,
    _socket: TcpSocket,
    request: Vec<u8>,
}

impl Connection {
    fn receive(&mut self, keep_alive: bool, work: usize, retained: &[u8]) -> bool {
        let Ok(bytes) = self.input.read(16 * 1024) else {
            return false;
        };
        self.request.extend_from_slice(&bytes);
        while let Some(end) = self.request.windows(4).position(|s| s == b"\r\n\r\n") {
            let header = &self.request[..end];
            let Ok(text) = std::str::from_utf8(header) else {
                return false;
            };
            let mut lines = text.split("\r\n");
            let first = lines.next().unwrap_or_default();
            if !first.starts_with("GET ") || !first.ends_with(" HTTP/1.1") {
                return false;
            }
            let mut close = !keep_alive;
            for line in lines {
                let Some((name, value)) = line.split_once(':') else {
                    return false;
                };
                if name.eq_ignore_ascii_case("connection") {
                    close |= value
                        .split(',')
                        .any(|v| v.trim().eq_ignore_ascii_case("close"));
                }
                // This fixture accepts only bodyless GETs, never production traffic.
                if name.eq_ignore_ascii_case("transfer-encoding")
                    || (name.eq_ignore_ascii_case("content-length") && value.trim() != "0")
                {
                    return false;
                }
            }
            if first.starts_with("GET /work ") {
                let mut value = black_box(0x123456789abcdef0_u64);
                for _ in 0..work {
                    value ^= value << 13;
                    value ^= value >> 7;
                    value ^= value << 17;
                }
                black_box(value);
            }
            black_box(retained);
            let response: &[u8] = if close {
                b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\nConnection: close\r\n\r\nok\n"
            } else {
                b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\nConnection: keep-alive\r\n\r\nok\n"
            };
            // Small, eagerly consumed benchmark responses; flush before dropping a
            // closing socket. Input multiplexing keeps idle clients from blocking.
            if self.output.blocking_write_and_flush(response).is_err() || close {
                return false;
            }
            self.request.drain(..end + 4);
        }
        self.request.len() <= 16 * 1024
    }
}

fn finish(mut operation: impl FnMut() -> Result<(), ErrorCode>, socket: &TcpSocket) {
    loop {
        match operation() {
            Ok(()) => return,
            Err(ErrorCode::WouldBlock) => socket.subscribe().block(),
            Err(error) => panic!("socket setup: {error:?}"),
        }
    }
}

fn main() {
    let mut retained = vec![0u8; number("BENCH_MEMORY_MIB", 0) * 1024 * 1024];
    for (index, byte) in retained.iter_mut().enumerate() {
        *byte = (index % 251) as u8;
    }
    black_box(&retained);
    let work = number("BENCH_WORK", 200_000);
    let keep_alive = number("BENCH_KEEPALIVE", 0) != 0;
    let network = instance_network();
    let listener = create_tcp_socket(IpAddressFamily::Ipv4).unwrap();
    listener
        .start_bind(
            &network,
            IpSocketAddress::Ipv4(Ipv4SocketAddress {
                port: number("PORT", 8080) as u16,
                address: (0, 0, 0, 0),
            }),
        )
        .unwrap();
    finish(|| listener.finish_bind(), &listener);
    listener.start_listen().unwrap();
    finish(|| listener.finish_listen(), &listener);
    let pending = listener.subscribe();
    let mut connections = Vec::<Connection>::new();
    loop {
        let mut waiters = vec![&pending];
        waiters.extend(connections.iter().map(|c| &c.readable));
        let ready = poll(&waiters);
        drop(waiters);
        // Descending removal preserves every ready index, including pipelined IO.
        let mut readable: Vec<_> = ready.iter().filter(|&&i| i != 0).copied().collect();
        readable.sort_unstable_by(|a, b| b.cmp(a));
        for index in readable {
            let index = index as usize - 1;
            if !connections[index].receive(keep_alive, work, &retained) {
                connections.remove(index);
            }
        }
        if ready.contains(&0) {
            loop {
                match listener.accept() {
                    Ok((socket, input, output)) => connections.push(Connection {
                        readable: input.subscribe(),
                        input,
                        output,
                        _socket: socket,
                        request: Vec::with_capacity(1024),
                    }),
                    Err(ErrorCode::WouldBlock) => break,
                    Err(ErrorCode::ConnectionAborted) => continue,
                    Err(error) => panic!("accept: {error:?}"),
                }
            }
        }
    }
}

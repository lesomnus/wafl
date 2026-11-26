//! Ordinary TCP sockets, with no platform SDK. Runs natively and on WASIp2.
//! A deliberately small polling loop for this example, not a production reactor.
use std::{
    io::{self, Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    time::{Duration, Instant},
};

struct Connection {
    stream: TcpStream,
    output: Vec<u8>,
    offset: usize,
    eof: bool,
    ready: Instant,
}

impl Connection {
    fn new(stream: TcpStream) -> io::Result<Self> {
        stream.set_nonblocking(true)?;
        let port = stream.local_addr()?.port();
        let version = std::env::var("GREETING").unwrap_or_else(|_| "hello".into());
        Ok(Self {
            stream,
            output: format!("{version}:{port}\n").into_bytes(),
            offset: 0,
            eof: false,
            ready: Instant::now(),
        })
    }

    // Return false after sending the response to the client's write-half EOF.
    // Backpressure bounds pending output to one 8 KiB chunk per connection.
    fn poll(&mut self) -> io::Result<bool> {
        if Instant::now() < self.ready {
            return Ok(true);
        }
        if self.offset < self.output.len() {
            match self.stream.write(&self.output[self.offset..]) {
                Ok(0) => return Ok(false),
                Ok(n) => self.offset += n,
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(true),
                Err(e) => return Err(e),
            }
            return Ok(true);
        }
        if self.eof {
            self.stream.shutdown(Shutdown::Write)?;
            return Ok(false);
        }
        let mut input = [0; 8192];
        match self.stream.read(&mut input) {
            Ok(n) => {
                self.output.clear();
                self.offset = 0;
                if n == 0 {
                    self.eof = true;
                    self.output.extend_from_slice(b"done\n");
                    // Optional simulated work after EOF, useful when checking
                    // that proxies preserve a response to a half-closed client.
                    self.ready = Instant::now()
                        + Duration::from_millis(
                            std::env::var("FIN_DELAY_MS")
                                .ok()
                                .and_then(|v| v.parse().ok())
                                .unwrap_or(0),
                        );
                } else {
                    self.output.extend_from_slice(&input[..n]);
                }
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
            Err(e) => return Err(e),
        }
        Ok(true)
    }
}

fn port(name: &str, default: u16) -> u16 {
    std::env::var(name).map_or(default, |v| v.parse().expect("port must be a u16"))
}

fn main() -> io::Result<()> {
    let mut listeners = Vec::new();
    for port in [port("PORT", 8080), port("SECOND_PORT", 8081)] {
        if port != 0 {
            let listener = TcpListener::bind(("0.0.0.0", port))?;
            listener.set_nonblocking(true)?;
            listeners.push(listener);
        }
    }
    let mut connections: Vec<Connection> = Vec::new();
    loop {
        for listener in &listeners {
            // Bound admission work and live connections in the example as well.
            for _ in 0..16 {
                match listener.accept() {
                    Ok((stream, _)) => {
                        if connections.len() < 128 {
                            connections.push(Connection::new(stream)?);
                        }
                    }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                    Err(e) => return Err(e),
                }
            }
        }
        connections.retain_mut(|c| c.poll().unwrap_or(false));
        std::thread::sleep(Duration::from_millis(1));
    }
}

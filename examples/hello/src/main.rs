use std::{
    io::{Read, Write},
    net::TcpListener,
};

fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".into());
    let name = std::env::var("GREETING").unwrap_or_else(|_| "Hello from Wafl".into());
    let listener = TcpListener::bind(format!("0.0.0.0:{port}"))?;
    let mut count = 0;
    for stream in listener.incoming() {
        let mut stream = stream?;
        let mut request = Vec::new();
        let mut chunk = [0; 1024];
        while !request.windows(4).any(|w| w == b"\r\n\r\n") {
            let n = stream.read(&mut chunk)?;
            if n == 0 {
                break;
            }
            request.extend_from_slice(&chunk[..n]);
            if request.len() > 16 * 1024 {
                break;
            }
        }
        if request.is_empty() {
            continue;
        }
        count += 1;
        let body = format!("{name}; request {count}\n");
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )?;
    }
    Ok(())
}

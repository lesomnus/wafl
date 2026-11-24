use std::{
    io::{Read, Write},
    net::TcpListener,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database = redis::Client::open(std::env::var("DATABASE_URL")?)?;
    let key = std::env::var("COUNTER_KEY").unwrap_or_else(|_| "wafl:visits".into());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".into());
    let listener = TcpListener::bind(format!("0.0.0.0:{port}"))?;
    println!("listening on port {port}");
    for stream in listener.incoming() {
        let mut stream = stream?;
        let mut request = Vec::new();
        let mut byte = [0];
        while !request.ends_with(b"\r\n\r\n") && request.len() < 16 * 1024 {
            if stream.read(&mut byte)? == 0 {
                break;
            }
            request.push(byte[0]);
        }
        if request.is_empty() {
            continue;
        }
        // The same client and DATABASE_URL work in a native process and WASI.
        // Redis owns the counter, so it survives instance Stop/Start.
        let count = database
            .get_connection()
            .and_then(|mut connection| redis::cmd("INCR").arg(&key).query::<u64>(&mut connection));
        let (status, body) = match count {
            Ok(count) => ("200 OK", format!("visits: {count}\n")),
            Err(_) => {
                // Do not expose the connection string or credentials in errors.
                eprintln!("database request failed");
                ("502 Bad Gateway", "database unavailable\n".into())
            }
        };
        write!(
            stream,
            "HTTP/1.1 {status}\r\nContent-Length: {}\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )?;
    }
    Ok(())
}

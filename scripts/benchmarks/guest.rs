//! Socket HTTP guest used unchanged by all resource benchmark hosts.
use std::{
    env,
    hint::black_box,
    io::{Read, Write},
    net::TcpListener,
};

fn number(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .map(|value| value.parse().unwrap())
        .unwrap_or(default)
}

fn main() -> std::io::Result<()> {
    let memory = number("BENCH_MEMORY_MIB", 0) * 1024 * 1024;
    // Fill and retain the allocation so every page is physically touched.
    let mut retained = vec![0u8; memory];
    for (index, byte) in retained.iter_mut().enumerate() {
        *byte = (index % 251) as u8;
    }
    black_box(&retained);
    let work = number("BENCH_WORK", 200_000);
    let listener = TcpListener::bind(("0.0.0.0", number("PORT", 8080) as u16))?;
    for stream in listener.incoming() {
        let mut stream = stream?;
        let mut request = Vec::with_capacity(1024);
        let mut buffer = [0; 1024];
        while !request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
            let read = stream.read(&mut buffer)?;
            if read == 0 || request.len() + read > 16 * 1024 {
                break;
            }
            request.extend_from_slice(&buffer[..read]);
        }
        if request.starts_with(b"GET /work ") {
            let mut value = black_box(0x123456789abcdef0_u64);
            for _ in 0..work {
                value ^= value << 13;
                value ^= value >> 7;
                value ^= value << 17;
            }
            black_box(value);
        }
        black_box(&retained);
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\nConnection: close\r\n\r\nok\n")?;
    }
    Ok(())
}

use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    time::Duration,
};

fn main() -> std::io::Result<()> {
    println!("guest boot");
    eprintln!("guest stderr");
    match std::env::var("MODE").as_deref() {
        Ok("exit") => return Ok(()),
        Ok("spin") => loop {
            std::hint::spin_loop();
        },
        Ok("delay") => std::thread::sleep(Duration::from_millis(200)),
        Ok("memory") => {
            let mut bytes = Vec::with_capacity(256 * 1024 * 1024);
            bytes.resize(256 * 1024 * 1024, 42u8);
            std::hint::black_box(bytes);
        }
        _ => {}
    }
    let listener = TcpListener::bind(("0.0.0.0", 8080))?;
    assert_eq!(listener.local_addr()?.port(), 8080);
    if std::env::var("MODE").as_deref() == Ok("close") {
        drop(listener);
        loop {
            std::hint::spin_loop();
        }
    }
    let mut count = 0;
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
        count += 1;
        let path = String::from_utf8_lossy(&request)
            .split_whitespace()
            .nth(1)
            .unwrap_or("/")
            .to_owned();
        let body = match path.as_str() {
            "/logs" => {
                println!("request stdout");
                eprintln!("request stderr");
                "logged".into()
            }
            "/data" => std::fs::read_to_string("/data/index.html")?,
            "/headers" => {
                let headers = String::from_utf8_lossy(&request).to_ascii_lowercase();
                assert!(headers.contains("host: same.example\r\n"));
                assert!(!headers.contains("wafl-instance:"));
                "preserved".into()
            }
            "/db" => {
                let mut db = TcpStream::connect(("127.0.0.1", 5432))?;
                assert_eq!(db.peer_addr()?, "127.0.0.1:5432".parse().unwrap());
                db.write_all(b"ping")?;
                let mut reply = [0; 4];
                db.read_exact(&mut reply)?;
                String::from_utf8_lossy(&reply).into_owned()
            }
            "/denied" => {
                assert!(TcpStream::connect(("127.0.0.1", 12345)).is_err());
                assert!(TcpListener::bind(("0.0.0.0", 9090)).is_err());
                assert!(std::fs::read("/etc/passwd").is_err());
                assert!(std::fs::read("/data/../main.wasm").is_err());
                assert!(std::fs::write("/data/index.html", "changed").is_err());
                assert!(std::fs::write("/data/new.txt", "changed").is_err());
                assert!(std::env::var("HOME").is_err());
                "denied".into()
            }
            _ => format!("{}:{count}", std::env::var("GREETING").unwrap_or_default()),
        };
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )?;
        if path == "/slow" {
            std::thread::sleep(Duration::from_millis(400));
        }
        let _ = stream.write_all(body.as_bytes());
    }
    Ok(())
}

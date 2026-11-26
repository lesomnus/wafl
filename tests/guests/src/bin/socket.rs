use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs, UdpSocket},
    time::Duration,
};

fn main() -> std::io::Result<()> {
    println!("guest boot");
    eprintln!("guest stderr");
    match std::env::var("MODE").as_deref() {
        Ok("secret-error") => return secret_error(),
        Ok("exit") => return Ok(()),
        Ok("spin") => loop {
            std::hint::spin_loop();
        },
        Ok("delay") => std::thread::sleep(Duration::from_millis(
            std::env::var("DELAY_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(200),
        )),
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
        if path == "/ready" && std::env::var("READY").as_deref() == Ok("false") {
            let _ = stream.write_all(b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
            continue;
        }
        let body = match path.as_str() {
            "/secret-error" => return secret_error(),
            "/exit" => return Ok(()),
            "/logs" => {
                println!("request stdout");
                eprintln!("request stderr");
                "logged".into()
            }
            "/data" => std::fs::read_to_string("/data/index.html")?,
            "/local" => stream.local_addr()?.port().to_string(),
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
            "/outbound" => {
                let address = std::env::var("BACKEND").unwrap();
                // Resolve inside the guest, then connect without a Binding.
                let addresses: Vec<_> = address.to_socket_addrs()?.collect();
                assert!(!addresses.is_empty());
                let mut backend = TcpStream::connect(addresses.as_slice())?;
                assert!(addresses.contains(&backend.peer_addr()?));
                backend.write_all(b"ping")?;
                let mut reply = [0; 4];
                backend.read_exact(&mut reply)?;
                String::from_utf8_lossy(&reply).into_owned()
            }
            "/bound" => bound_connect(std::env::var("BACKEND").unwrap().parse().unwrap()),
            "/denied" => {
                assert!(TcpListener::bind(("0.0.0.0", 9090)).is_err());
                assert!(TcpListener::bind(("0.0.0.0", 0)).is_err());
                assert!(TcpListener::bind(("::", 0)).is_err());
                assert!(UdpSocket::bind(("127.0.0.1", 0)).is_err());
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

fn secret_error() -> std::io::Result<()> {
    let value = std::env::var("GREETING").unwrap();
    eprintln!("{value}");
    Err(std::io::Error::other(value))
}

fn bound_connect(address: SocketAddr) -> String {
    use wasip2::sockets::{
        instance_network::instance_network,
        network::{
            ErrorCode, IpAddressFamily, IpSocketAddress, Ipv4SocketAddress, Ipv6SocketAddress,
        },
        tcp_create_socket::create_tcp_socket,
    };
    let (family, local, remote) = match address {
        SocketAddr::V4(addr) => {
            let [a, b, c, d] = addr.ip().octets();
            (
                IpAddressFamily::Ipv4,
                IpSocketAddress::Ipv4(Ipv4SocketAddress {
                    port: 0,
                    address: (0, 0, 0, 0),
                }),
                IpSocketAddress::Ipv4(Ipv4SocketAddress {
                    port: addr.port(),
                    address: (a, b, c, d),
                }),
            )
        }
        SocketAddr::V6(addr) => {
            let [a, b, c, d, e, f, g, h] = addr.ip().segments();
            (
                IpAddressFamily::Ipv6,
                IpSocketAddress::Ipv6(Ipv6SocketAddress {
                    port: 0,
                    flow_info: 0,
                    address: (0, 0, 0, 0, 0, 0, 0, 0),
                    scope_id: 0,
                }),
                IpSocketAddress::Ipv6(Ipv6SocketAddress {
                    port: addr.port(),
                    flow_info: 0,
                    address: (a, b, c, d, e, f, g, h),
                    scope_id: 0,
                }),
            )
        }
    };
    let network = instance_network();
    let socket = create_tcp_socket(family).unwrap();
    let poll = socket.subscribe();
    socket.start_bind(&network, local).unwrap();
    loop {
        match socket.finish_bind() {
            Err(ErrorCode::WouldBlock) => poll.block(),
            result => {
                result.unwrap();
                break;
            }
        }
    }
    match socket.local_address().unwrap() {
        IpSocketAddress::Ipv4(addr) => assert_eq!(addr.address, (0, 0, 0, 0)),
        IpSocketAddress::Ipv6(addr) => assert_eq!(addr.address, (0, 0, 0, 0, 0, 0, 0, 0)),
    }
    socket.start_connect(&network, remote).unwrap();
    let (input, output) = loop {
        match socket.finish_connect() {
            Err(ErrorCode::WouldBlock) => poll.block(),
            result => break result.unwrap(),
        }
    };
    output.blocking_write_and_flush(b"ping").unwrap();
    let mut reply = Vec::new();
    while reply.len() < 4 {
        reply.extend(input.blocking_read((4 - reply.len()) as u64).unwrap());
    }
    String::from_utf8(reply).unwrap()
}

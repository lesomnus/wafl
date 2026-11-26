//! WASI socket translation stays in the host; guests use standard WASI APIs.
use crate::config::Spec;
use crate::endpoint::{Claim, Ports};
use crate::{logs::Logs, proto::wafl::Channel};
use anyhow::{Context, Result};
use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddr},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::sync::oneshot;
use wasmtime::{
    Config, Engine, Store, StoreContextMut, StoreLimits, StoreLimitsBuilder,
    component::{Component, Linker, Resource, ResourceTable, ResourceType},
};
use wasmtime_wasi::{
    FsPerms, WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView,
    p2::{
        self, SocketResult, TcpSocket,
        bindings::{
            Command,
            sockets::{
                network::{ErrorCode, IpSocketAddress, Network},
                tcp::HostTcpSocket,
            },
        },
    },
    sockets::{SocketAddrUse, WasiSocketsView},
};

pub(crate) struct Runtime {
    pub engine: Engine,
    pub direct: Option<Arc<Ports>>,
    linker: Linker<State>,
    ticker: Option<std::thread::JoinHandle<()>>,
    stopped: Arc<AtomicBool>,
}

struct State {
    wasi: WasiCtx,
    table: ResourceTable,
    limits: StoreLimits,
    port: u16,
    bindings: HashMap<u16, SocketAddr>,
    remotes: HashMap<u32, SocketAddr>,
    ports: Vec<u16>,
    listeners: HashMap<u32, u16>,
    addresses: HashMap<u16, SocketAddr>,
    ready: Option<oneshot::Sender<Ready>>,
    direct: Option<Arc<Ports>>,
    claims: HashMap<u16, Arc<Claim>>,
    instance: [u8; 16],
    generation: u64,
}

pub(crate) struct Ready {
    pub address: SocketAddr,
    pub claims: Vec<Arc<Claim>>,
}

impl WasiView for State {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

// Preserve WASI error codes; malformed resource handles remain traps.
fn socket_result<T>(
    result: SocketResult<T>,
) -> wasmtime::Result<(std::result::Result<T, ErrorCode>,)> {
    Ok((match result {
        Ok(v) => Ok(v),
        Err(e) => Err(e.downcast()?),
    },))
}

impl Runtime {
    pub fn new(direct: Option<Arc<Ports>>) -> Result<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true).epoch_interruption(true);
        let engine = Engine::new(&config)?;
        let mut linker = Linker::<State>::new(&engine);
        p2::add_to_linker_async(&mut linker)?;
        linker.allow_shadowing(true);
        // Match the pinned wasmtime-wasi WIT version. The linker also resolves
        // compatible earlier 0.2 imports against these definitions.
        let mut tcp = linker.instance("wasi:sockets/tcp@0.2.12")?;
        tcp.resource(
            "tcp-socket",
            ResourceType::host::<TcpSocket>(),
            |mut cx, rep| {
                let state = cx.data_mut();
                state.remotes.remove(&rep);
                state.sockets().drop(Resource::new_own(rep))?;
                // The declared listener is the instance's lifetime boundary. Retiring
                // it must also retire the route, even if the guest keeps computing.
                if state.listeners.contains_key(&rep) {
                    return Err(wasmtime::Error::msg("guest closed ingress listener"));
                }
                Ok(())
            },
        )?;
        tcp.func_wrap_async(
            "[method]tcp-socket.start-bind",
            |mut cx: StoreContextMut<'_, State>,
             (socket, network, guest): (
                Resource<TcpSocket>,
                Resource<Network>,
                IpSocketAddress,
            )| {
                Box::new(async move {
                    let guest: SocketAddr = guest.into();
                    let state = cx.data_mut();
                    // One listener per declared port. Port zero is allowed for explicit
                    // client-side binds but may never become an ingress listener.
                    if !(guest.ip().is_loopback() || guest.ip().is_unspecified())
                        || (!state.ports.contains(&guest.port()) && guest.port() != 0)
                        || (state.ports.contains(&guest.port()) && !guest.is_ipv4())
                    {
                        return Ok((Err(ErrorCode::AccessDenied),));
                    }
                    if state.ports.contains(&guest.port())
                        && state.listeners.values().any(|p| *p == guest.port())
                    {
                        return Ok((Err(ErrorCode::AddressInUse),));
                    }
                    let rep = socket.rep();
                    if state.ports.contains(&guest.port())
                        && let Some(pool) = state.direct.clone()
                    {
                        // Reserve durably before bind. Never probe and close an
                        // OS socket; the WASI socket performs the actual bind.
                        for _ in 0..pool.count() {
                            let claim = pool
                                .reserve(state.instance, state.generation)
                                .await
                                .map_err(|e| wasmtime::Error::msg(format!("{e:#}")))?;
                            match state
                                .sockets()
                                .start_bind(
                                    Resource::new_borrow(rep),
                                    Resource::new_borrow(network.rep()),
                                    claim.bind.into(),
                                )
                                .await
                            {
                                Ok(()) => {
                                    state.listeners.insert(rep, guest.port());
                                    state.claims.insert(guest.port(), claim);
                                    return Ok((Ok(()),));
                                }
                                Err(error) => {
                                    let code = error.downcast()?;
                                    // A bind failure was never advertised; its
                                    // claim can be safely discarded.
                                    claim
                                        .release()
                                        .await
                                        .map_err(|e| wasmtime::Error::msg(format!("{e:#}")))?;
                                    if code != ErrorCode::AddressInUse {
                                        return Ok((Err(code),));
                                    }
                                }
                            }
                        }
                        return Ok((Err(ErrorCode::AddressInUse),));
                    }
                    // Keep client binds in their original address family and do
                    // not force wildcard clients onto the host loopback route.
                    let host = if state.ports.contains(&guest.port()) {
                        SocketAddr::from((Ipv4Addr::LOCALHOST, 0))
                    } else {
                        guest
                    };
                    let result = state
                        .sockets()
                        .start_bind(socket, network, host.into())
                        .await;
                    if result.is_ok() && state.ports.contains(&guest.port()) {
                        state.listeners.insert(rep, guest.port());
                    }
                    socket_result(result)
                })
            },
        )?;
        tcp.func_wrap_async(
            "[method]tcp-socket.start-listen",
            |mut cx: StoreContextMut<'_, State>, (socket,): (Resource<TcpSocket>,)| {
                Box::new(async move {
                    let state = cx.data_mut();
                    if !state.listeners.contains_key(&socket.rep()) {
                        return Ok((Err(ErrorCode::AccessDenied),));
                    }
                    socket_result(state.sockets().start_listen(socket).await)
                })
            },
        )?;
        tcp.func_wrap(
            "[method]tcp-socket.finish-listen",
            |mut cx: StoreContextMut<'_, State>, (socket,): (Resource<TcpSocket>,)| {
                let state = cx.data_mut();
                let rep = socket.rep();
                let result = state.sockets().finish_listen(socket);
                if result.is_ok() {
                    let address = state.sockets().local_address(Resource::new_borrow(rep));
                    let address = match address {
                        Ok(a) => SocketAddr::from(a),
                        Err(e) => return socket_result(Err(e)),
                    };
                    let port = state.listeners[&rep];
                    state.addresses.insert(port, address);
                    if state.addresses.len() == state.ports.len()
                        && let Some(ready) = state.ready.take()
                    {
                        let _ = ready.send(Ready {
                            address: state.addresses[&state.port],
                            claims: state
                                .ports
                                .iter()
                                .filter_map(|p| state.claims.get(p).cloned())
                                .collect(),
                        });
                    }
                }
                socket_result(result)
            },
        )?;
        tcp.func_wrap(
            "[method]tcp-socket.local-address",
            |mut cx: StoreContextMut<'_, State>, (socket,): (Resource<TcpSocket>,)| {
                let state = cx.data_mut();
                let listener = state.listeners.get(&socket.rep()).copied();
                let result = state.sockets().local_address(socket).map(|address| {
                    let mut address: SocketAddr = address.into();
                    let port = listener.or_else(|| {
                        state
                            .addresses
                            .iter()
                            .find(|(_, host)| host.port() == address.port())
                            .map(|(port, _)| *port)
                    });
                    if let Some(port) = port {
                        address.set_port(port);
                    }
                    IpSocketAddress::from(address)
                });
                socket_result(result)
            },
        )?;
        tcp.func_wrap(
            "[method]tcp-socket.start-connect",
            |mut cx: StoreContextMut<'_, State>,
             (socket, network, guest): (
                Resource<TcpSocket>,
                Resource<Network>,
                IpSocketAddress,
            )| {
                let guest: SocketAddr = guest.into();
                let state = cx.data_mut();
                let target = (guest.is_ipv4() && guest.ip().is_loopback())
                    .then(|| state.bindings.get(&guest.port()).copied())
                    .flatten()
                    .unwrap_or(guest);
                let rep = socket.rep();
                let result = state
                    .sockets()
                    .start_connect(socket, network, target.into());
                if result.is_ok() && target != guest {
                    state.remotes.insert(rep, guest);
                }
                socket_result(result)
            },
        )?;
        tcp.func_wrap(
            "[method]tcp-socket.remote-address",
            |mut cx: StoreContextMut<'_, State>, (socket,): (Resource<TcpSocket>,)| {
                let state = cx.data_mut();
                let guest = state.remotes.get(&socket.rep()).copied();
                socket_result(
                    state
                        .sockets()
                        .remote_address(socket)
                        .map(|address| guest.map(IpSocketAddress::from).unwrap_or(address)),
                )
            },
        )?;
        linker.allow_shadowing(false);
        let stopped = Arc::new(AtomicBool::new(false));
        let flag = stopped.clone();
        let clock = engine.clone();
        // A dedicated clock still advances when all async executor threads are
        // executing CPU-bound guests. Epoch yields make cancellation responsive.
        let ticker = std::thread::Builder::new()
            .name("wafl-epoch".into())
            .spawn(move || {
                while !flag.load(Ordering::Relaxed) {
                    std::thread::sleep(Duration::from_millis(10));
                    clock.increment_epoch();
                }
            })?;
        Ok(Self {
            engine,
            direct,
            linker,
            ticker: Some(ticker),
            stopped,
        })
    }

    pub async fn execute(
        &self,
        component: &Component,
        app: &Spec,
        data: &std::path::Path,
        logs: Arc<Logs>,
        generation: u64,
        ready: oneshot::Sender<Ready>,
    ) -> Result<()> {
        let bindings: HashMap<_, _> = app.bindings.iter().map(|b| (b.port, b.target)).collect();
        let mut wasi = WasiCtxBuilder::new();
        wasi.stdout(logs.sink(generation, Channel::Stdout));
        wasi.stderr(logs.sink(generation, Channel::Stderr));
        wasi.preopened_dir(data, "/data", FsPerms::ReadOnly)?;
        wasi.arg("main.wasm")
            .env("PORT", app.port.to_string())
            .allow_tcp(true)
            .allow_udp(false)
            .allow_ip_name_lookup(true);
        for (key, value) in &app.manifest.env {
            wasi.env(key, value);
        }
        for (key, value) in &app.secrets.0 {
            wasi.env(key, value);
        }
        let direct = self.direct.as_ref().map(|p| p.config.clone());
        wasi.socket_addr_check(move |addr, reason| {
            let allowed = match reason {
                SocketAddrUse::TcpBind => {
                    ((addr.ip().is_loopback() || addr.ip().is_unspecified()) && addr.port() == 0)
                        || direct.as_ref().is_some_and(|d| {
                            addr.ip() == std::net::IpAddr::V4(d.listen)
                                && (d.first..=d.last).contains(&addr.port())
                        })
                }
                SocketAddrUse::TcpListen => {
                    addr.ip().is_loopback()
                        || direct.as_ref().is_some_and(|d| {
                            addr.ip() == std::net::IpAddr::V4(d.listen)
                                && (d.first..=d.last).contains(&addr.port())
                        })
                }
                // For accept this address is the PEER, not the local listener.
                SocketAddrUse::TcpAccept => {
                    addr.ip().is_loopback() || (direct.is_some() && addr.is_ipv4())
                }
                // Outbound TCP uses the runner's network reachability. Ingress
                // remains restricted to the one virtualized listener above.
                SocketAddrUse::TcpConnect => true,
                _ => false,
            };
            Box::pin(async move { allowed })
        });
        let mut table = ResourceTable::new();
        table.set_max_capacity(1024);
        let state = State {
            wasi: wasi.build(),
            table,
            limits: StoreLimitsBuilder::new()
                .memory_size(app.memory_bytes)
                .memories(1)
                .table_elements(65_536)
                .tables(8)
                .instances(32)
                .build(),
            port: app.port,
            bindings,
            remotes: HashMap::new(),
            ports: app.sockets.iter().map(|s| s.port as u16).collect(),
            listeners: HashMap::new(),
            addresses: HashMap::new(),
            ready: Some(ready),
            direct: self.direct.clone(),
            claims: HashMap::new(),
            instance: app.id,
            generation,
        };
        let mut store = Store::new(&self.engine, state);
        store.limiter(|state| &mut state.limits);
        store.set_epoch_deadline(1);
        store.epoch_deadline_async_yield_and_update(1);
        let command = Command::instantiate_async(&mut store, component, &self.linker)
            .await
            .map_err(anyhow::Error::from)
            .context("instantiate WASI command")?;
        command
            .wasi_cli_run()
            .call_run(&mut store)
            .await
            .map_err(anyhow::Error::from)
            .context("execute WASI command")?
            .map_err(|()| anyhow::anyhow!("guest returned failure: {}", logs.stderr(generation)))
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Relaxed);
        if let Some(ticker) = self.ticker.take() {
            let _ = ticker.join();
        }
    }
}

mod admission;
mod bundle;
mod compiler;
pub mod config;
pub mod control;
mod endpoint;
pub mod host;
mod logs;
pub mod proxy;
pub mod runner;
mod runtime;
mod store;
mod telemetry;

#[path = "../generated/messages/mod.rs"]
#[rustfmt::skip]
pub mod proto;
#[path = "../generated/services/mod.rs"]
#[rustfmt::skip]
#[allow(clippy::needless_lifetimes)]
pub mod services;

pub mod server {
    include!("../generated/server.rs");
}

mod bundle;
pub mod config;
pub mod control;
pub mod proxy;
pub mod runner;
mod runtime;

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

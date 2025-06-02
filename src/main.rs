use std::env;
use std::future::pending;
mod auth;
mod constants;
mod local;
mod plugin;
mod types;
mod utils;
use crate::local::service::LocalService;

use crate::plugin::dbus::{build_connection, register_plugin};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let version = env!("CARGO_PKG_VERSION");

    env::set_var("RUST_LOG", "debug");
    // Configure logging to output to stdout
    env_logger::builder()
        .target(env_logger::Target::Stdout)
        .init();

    log::info!("Starting Playtron Plugin version: {version}");

    let local_service = LocalService::new();
    build_connection(local_service).await?;
    register_plugin().await;

    // Do other things or go to wait forever
    pending::<()>().await;

    Ok(())
}

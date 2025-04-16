use std::env;
use std::future::pending;
mod constants;
mod local;
mod plugin;
mod types;
mod utils;

use crate::local::service::ExampleService;

use crate::plugin::dbus::{build_connection, register_plugin};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env::set_var("RUST_LOG", "debug");
    // Configure logging to output to stdout
    env_logger::builder()
        .target(env_logger::Target::Stdout)
        .init();

    log::info!("Starting Playtron Plugin");

    let example_service = ExampleService::new();
    build_connection(example_service).await?;
    register_plugin().await;

    // Do other things or go to wait forever
    pending::<()>().await;

    Ok(())
}

use crate::constants::{BUS_NAME, CLIENT_PATH};
use crate::local::service::ExampleService;
use crate::plugin;
use crate::types::results::EmptyResult;
use tokio::sync::Mutex;
use zbus::{connection, zvariant::ObjectPath, Connection};

lazy_static::lazy_static! {
    pub static ref CONNECTION: Mutex<Option<Connection>> =
        Mutex::new(None);

}

pub async fn build_connection(service: ExampleService) -> EmptyResult {
    let plugin = plugin::plugin_interface::Plugin {};
    let library_provider = plugin::library_provider::LibraryProvider::new(service.clone());

    *CONNECTION.lock().await = Some(
        connection::Builder::session()?
            .name(BUS_NAME)?
            .serve_at(CLIENT_PATH, plugin)?
            .serve_at(CLIENT_PATH, library_provider)?
            .build()
            .await?,
    );
    Ok(())
}

pub async fn register_plugin() {
    if let Some(conn) = CONNECTION.lock().await.as_ref() {
        if let Err(error) = conn
            .call_method(
                Some("one.playtron.Playserve"),
                "/one/playtron/plugins/Manager",
                Some("one.playtron.plugin.Manager"),
                "RegisterPlugin",
                &(BUS_NAME, ObjectPath::try_from(CLIENT_PATH).unwrap()),
            )
            .await
        {
            log::error!("The plugin failed to register: {}", error);
        } else {
            log::info!("The plugin registered successfully");
        }
    }
}

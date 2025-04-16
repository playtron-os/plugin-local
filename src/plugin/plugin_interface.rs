use crate::constants::{MINIMUM_API_VERSION, NAME, PLUGIN_ID, VERSION};
use zbus::fdo;
use zbus_macros::interface;

pub struct Plugin {}

#[interface(name = "one.playtron.plugin.Plugin")]
impl Plugin {
    /// unique plugin id of the plugin (e.g. “steam”)
    ///
    /// # Example
    ///
    /// busctl --user get-property one.playtron.EpicGames \
    ///   /one/playtron/EpicGames/LegendaryClient0 \
    ///   one.playtron.plugin.Plugin \
    ///   Id
    #[zbus(property)]
    async fn id(&self) -> fdo::Result<&str> {
        Ok(PLUGIN_ID)
    }

    /// Human readable name of the plugin (e.g. “Steam”)
    ///
    /// busctl --user get-property one.playtron.EpicGames \
    ///   /one/playtron/EpicGames/LegendaryClient0 \
    ///   one.playtron.plugin.Plugin \
    ///   Name
    #[zbus(property)]
    async fn name(&self) -> fdo::Result<&str> {
        Ok(NAME)
    }

    /// version of the plugin (e.g. “0.1.0”)
    ///
    /// busctl --user get-property one.playtron.EpicGames \
    ///   /one/playtron/EpicGames/LegendaryClient0 \
    ///   one.playtron.plugin.Plugin \
    ///   Version
    #[zbus(property)]
    async fn version(&self) -> fdo::Result<&str> {
        Ok(VERSION)
    }

    /// the plugin API version that the plugin supports
    ///
    /// busctl --user get-property one.playtron.EpicGames \
    ///   /one/playtron/EpicGames/LegendaryClient0 \
    ///   one.playtron.plugin.Plugin \
    ///   MinimumApiVersion
    #[zbus(property)]
    async fn minimum_api_version(&self) -> fdo::Result<&str> {
        Ok(MINIMUM_API_VERSION)
    }
}

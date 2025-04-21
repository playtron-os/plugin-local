use crate::constants::{LIBRARY_PROVIDER_ID, LIBRARY_PROVIDER_NAME};
use crate::local::service::LocalService;
use crate::types::app::{DownloadStage, EulaEntry, InstalledApp, LaunchOption, ProviderItem};
use crate::types::cloud_sync::CloudPath;
use std::collections::HashMap;
use zbus::fdo;
use zbus::object_server::SignalEmitter;
use zbus_macros::interface;

pub struct LibraryProvider {
    pub service: LocalService,
}

impl LibraryProvider {
    pub fn new(service: LocalService) -> Self {
        LibraryProvider { service }
    }
}

#[interface(name = "one.playtron.plugin.LibraryProvider")]
impl LibraryProvider {
    /// Human readable name of the library provider
    #[zbus(property)]
    async fn name(&self) -> &str {
        LIBRARY_PROVIDER_NAME
    }

    /// Name of the provider which this plugins represents
    /// (has to be unique between all library provider plugins)
    #[zbus(property)]
    async fn provider(&self) -> &str {
        LIBRARY_PROVIDER_ID
    }

    #[zbus(property)]
    async fn install_target(&self) -> &str {
        "folder"
    }

    /// Emitted when a installed game has a new version available to update.
    #[zbus(signal)]
    pub async fn app_new_version_found(
        emitter: &SignalEmitter<'_>,
        app_id: String,
        version: String,
    ) -> zbus::Result<()>;

    /// Install progressed signal
    #[zbus(signal)]
    pub async fn install_progressed(
        emitter: &SignalEmitter<'_>,
        app_id: String,
        stage: DownloadStage,
        downloaded_bytes: u64,
        total_download_size: u64,
        progress: f64,
    ) -> zbus::Result<()>;

    /// Emitted when an install starts
    #[zbus(signal)]
    pub async fn install_started(
        emitter: &SignalEmitter<'_>,
        app_id: String,
        version: String,
        install_directory: String,
        total_download_size: u64,
        requires_internet_connection: bool,
        os: String,
    ) -> zbus::Result<()>;

    /// Emitted when a game’s installation step is completed.
    #[zbus(signal)]
    pub async fn install_completed(emitter: &SignalEmitter<'_>, value: String) -> zbus::Result<()>;

    /// Emitted when a game’s installation step is failed.
    #[zbus(signal)]
    pub async fn install_failed(
        emitter: &SignalEmitter<'_>,
        app_id: &str,
        error: &str,
    ) -> zbus::Result<()>;

    /// TODO Undocumented
    #[zbus(signal)]
    pub async fn installed_apps_updated(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;

    /// LaunchError signal emitted after calling pre_launch_hook
    #[zbus(signal)]
    pub async fn launch_error(
        emitter: &SignalEmitter<'_>,
        app_id: &str,
        error: &str,
    ) -> zbus::Result<()>;

    /// LaunchReady signal emitted after calling pre_launch_hook
    #[zbus(signal)]
    pub async fn launch_ready(emitter: &SignalEmitter<'_>, app_id: &str) -> zbus::Result<()>;

    /// Emitted when provider received an event about new library entries
    #[zbus(signal)]
    pub async fn library_updated(
        emitter: &SignalEmitter<'_>,
        new_items: &[ProviderItem],
    ) -> zbus::Result<()>;

    /// Emitted when a game has finished being moved to a new folder
    #[zbus(signal)]
    async fn move_item_completed(
        emitter: &SignalEmitter<'_>,
        app_id: String,
        install_folder: String,
    ) -> zbus::Result<()>;

    /// Fired when a game has failed to be moved to a new folder
    #[zbus(signal)]
    pub async fn move_item_failed(
        emitter: &SignalEmitter<'_>,
        app_id: String,
        error: String,
    ) -> zbus::Result<()>;

    /// Emitted during MoveItem, sending progress of the move.
    /// This signal is emitted to report the progress of a file move operation.
    /// The `app_id` parameter identifies the application being moved, and the `progress`
    /// parameter is a value between 0.0 and 1.0 indicating the percentage of the move
    /// that has been completed.
    #[zbus(signal)]
    pub async fn move_item_progressed(
        emitter: &SignalEmitter<'_>,
        app_id: String,
        progress: f32,
    ) -> zbus::Result<()>;

    /// Retrieve EULAs for an app
    async fn get_eulas(
        &self,
        app_id: &str,
        country: &str,
        locale: &str,
    ) -> fdo::Result<Vec<EulaEntry>> {
        match self.service.get_eulas(app_id, country, locale).await {
            Ok(eulas) => Ok(eulas),
            Err(e) => Err(fdo::Error::Failed(e.to_string())),
        }
    }

    /// Returns a list of available install options for the given provider app id.
    /// Install options are a list of arbitrary values that the library provider knows
    /// about which can affect what version of the app is installed and how it is installed
    /// (e.g. “version”, “language”, “low-violence”, “branch”, etc.)
    ///
    /// # Example
    ///
    /// busctl --user call one.playtron.EpicGames \
    ///   /one/playtron/EpicGames/LegendaryClient0 \
    ///   one.playtron.plugin.LibraryProvider \
    ///   GetInstallOptions "s" "460950"
    async fn get_install_options(
        &self,
        app_id: &str,
    ) -> fdo::Result<Vec<(String, String, Vec<String>)>> {
        log::info!("get_install_options for {}", app_id);
        Ok(Vec::new())
    }

    /// busctl --user call one.playtron.BasePlugin \
    ///   /one/playtron/BasePlugin/PluginClient0 \
    ///   one.playtron.plugin.LibraryProvider \
    ///   GetInstalledApps
    async fn get_installed_apps(&self) -> fdo::Result<Vec<InstalledApp>> {
        log::info!("get_installed_apps");
        match self.service.get_installed_apps().await {
            Ok(apps) => Ok(apps),
            Err(err) => Err(fdo::Error::Failed(err.to_string())),
        }
    }

    /// Returns a provider item for a given provider app id.
    ///
    /// # Example
    ///
    /// busctl --user call one.playtron.EpicGames \
    ///   /one/playtron/EpicGames/LegendaryClient0 \
    ///   one.playtron.plugin.LibraryProvider \
    ///   GetProviderItem "s" "460950"
    async fn get_provider_item(&self, app_id: String) -> fdo::Result<ProviderItem> {
        self.service.get_provider_item(&app_id).await
    }

    /// Returns all provider items.
    ///
    /// # Example
    ///
    /// busctl --user call one.playtron.EpicGames \
    ///   /one/playtron/EpicGames/LegendaryClient0 \
    ///   one.playtron.plugin.LibraryProvider \
    ///   GetProviderItems
    async fn get_provider_items(
        &self,
        //#[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) -> fdo::Result<Vec<ProviderItem>> {
        log::info!("Getting provider items");
        let new_items = self.service.get_provider_items().await?;
        //LibraryProviderSignals::library_updated(&emitter, &new_items).await?;
        Ok(new_items)
    }

    /// Return the components to be installed after the game is downloaded
    async fn get_post_install_steps(&self, app_id: &str) -> fdo::Result<String> {
        self.service.get_post_install_steps(app_id).await
    }

    /// Returns a list of available launch options. This includes information
    /// on how the game should be launched together with other useful metadata.
    ///
    /// # Example
    ///
    /// busctl --user call one.playtron.EpicGames \
    ///   /one/playtron/EpicGames/LegendaryClient0 \
    ///   one.playtron.plugin.LibraryProvider \
    ///   GetLaunchOptions "s" "460950"
    async fn get_launch_options(&self, app_id: &str) -> fdo::Result<Vec<LaunchOption>> {
        self.service.get_launch_options(app_id).await
    }

    async fn get_item_metadata(&self, app_id: &str) -> fdo::Result<String> {
        Ok(self.service.get_item_metadata(app_id).await)
    }

    /// Start installing the app with the given provider app id to the given
    /// block device. It is the plugin’s responsibility to determine where on
    /// the mounted disk the app will be installed. Available install options
    /// can be queried by calling the GetInstallOptions method. Once the
    /// install has started, the plugin will be responsible for sending
    /// InstallProgressed signals about the progress of the install.
    ///
    /// # Example
    ///
    /// busctl --user call one.playtron.BasePlugin \
    ///   /one/playtron/BasePlugin/PluginClient0 \
    ///   one.playtron.plugin.LibraryProvider \
    ///   Install "ssa{sv}" "Cinebench" "/dev/sda1" 2 'os' s windows 'language' s english
    async fn install(
        &self,
        _app_id: &str,
        _dest_path: &str,
        _options: HashMap<String, zbus::zvariant::Value<'_>>,
    ) -> fdo::Result<i32> {
        Err(fdo::Error::Failed("Install is not supported".to_string()))
    }

    /// Moves the given app to another disk and returns the new install directory.
    ///
    /// # Example
    ///
    /// busctl --user call one.playtron.EpicGames \
    ///   /one/playtron/EpicGames/LegendaryClient0 \
    ///   one.playtron.plugin.LibraryProvider \
    ///   MoveItem "ss" "460950" "/dev/sda3"
    async fn move_item(&self, app_id: &str, dest_path: &str) -> fdo::Result<()> {
        self.service.move_item(app_id, dest_path).await
    }

    /// Uninstalls the given app.
    ///
    /// # Example
    ///
    /// busctl --user call one.playtron.EpicGames \
    ///   /one/playtron/EpicGames/LegendaryClient0 \
    ///   one.playtron.plugin.LibraryProvider \
    ///   Uninstall "s" "460950"
    async fn uninstall(&self, app_id: &str) -> fdo::Result<()> {
        self.service.uninstall(app_id).await
    }

    /// Start updating the given game. Once the update starts,
    /// the plugin will be responsible for sending signals
    /// about the progress of the update.
    ///
    /// # Example
    ///
    /// busctl --user call one.playtron.EpicGames \
    ///   /one/playtron/EpicGames/LegendaryClient0 \
    ///   one.playtron.plugin.LibraryProvider \
    ///   Update "s" "460950"
    async fn update(&self, _app_id: String) -> fdo::Result<()> {
        Err(fdo::Error::Failed("Update is not supported".to_string()))
    }

    /// Obtain the list of CloudPaths applicable for given app_id and platform, if the list is empty it is assumed saves are not supported
    ///
    /// # Example
    ///
    /// busctl --user call one.playtron.EpicGames \
    ///   /one/playtron/EpicGames/LegendaryClient0 \
    ///   one.playtron.plugin.LibraryProvider \
    ///   GetSavePathPatterns ss 730 windows
    async fn get_save_path_patterns(
        &self,
        app_id: &str,
        platform: &str,
    ) -> fdo::Result<Vec<CloudPath>> {
        self.service.get_save_path_patterns(app_id, platform).await
    }

    fn pause_install(&self) {
        log::info!("pause install");
    }

    /// Trigger discovery of provider items. Should emit LibraryUpdated signal if new items are discovered.
    ///
    /// # Example
    ///
    /// busctl --user call one.playtron.EpicGames \
    ///   /one/playtron/EpicGames/LegendaryClient0 \
    ///   one.playtron.plugin.LibraryProvider \
    ///   Refresh
    fn refresh(&self) -> fdo::Result<()> {
        log::info!("refresh");
        Ok(())
    }

    /// Executed before a game is launched. Should emit LaunchReady signal if the game can be launched.
    async fn pre_launch_hook(
        &self,
        app_id: &str,
        using_offline_mode: bool,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) -> fdo::Result<Vec<String>> {
        let res = self
            .service
            .pre_launch_hook(app_id.to_string(), using_offline_mode)
            .await;
        emitter.launch_ready(app_id).await?;
        res
    }

    /// Executed after a game has launched
    async fn post_launch_hook(&self, app_id: &str) -> fdo::Result<()> {
        log::info!("post launch hook for {}", app_id);
        Ok(())
    }

    async fn sync_installed_apps(&self) -> fdo::Result<()> {
        log::info!("sync installed apps");
        Ok(())
    }

    async fn import(&self, app_id: &str, install_folder: &str) -> fdo::Result<()> {
        self.service.import(app_id, install_folder).await
    }
}

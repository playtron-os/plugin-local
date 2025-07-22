use crate::utils::date::optional_date_serializer;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use zbus::zvariant::{OwnedValue, SerializeDict, Type, Value};

#[derive(Serialize, Deserialize, Debug, Type, Clone)]
pub struct EulaEntry {
    pub id: String,
    pub name: String,
    pub version: i32,
    pub url: String,
    pub body: String,
    pub country: String,
    pub language: String,
}

#[derive(Serialize, Deserialize, Debug, Type)]
pub struct InstalledApp {
    pub app_id: String,
    pub installed_path: String,
    pub downloaded_bytes: u64,
    pub total_download_size: u64,
    pub disk_size: u64,
    pub version: String,
    pub latest_version: String,
    pub update_pending: bool,
    pub os: String,
    pub language: String,
    pub disabled_dlc: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Type)]
pub enum LaunchType {
    Unknown,
    Launcher,
    Game,
    Tool,
    Document,
    Other,
}

#[derive(Serialize, Deserialize, Display, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
/// Platforms supported to donwload an app
/// Right now only the steam downloader is using this to select different platforms
pub enum Platform {
    Linux,
    Windows,
    MacOS,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default, Type)]
/// There are more values for this enum such as DLC/Config, but we only care about the Game and Aplication
pub enum AppType {
    #[default]
    #[serde(alias = "game")]
    Game,
    #[serde(alias = "application")]
    Application,
    #[serde(alias = "tool")]
    Tool,
    #[serde(alias = "DLC", alias = "dlc")]
    Dlc,
    #[serde(alias = "music")]
    Music,
    #[serde(alias = "config")]
    Config,
    #[serde(alias = "demo")]
    Demo,
    #[serde(alias = "beta")]
    Beta,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default, Type)]
pub enum UpdateStage {
    #[default]
    #[serde(alias = "none")]
    None,
    #[serde(alias = "error")]
    Error,
    #[serde(alias = "preallocating")]
    Preallocating,
    #[serde(alias = "downloading")]
    Downloading,
    #[serde(alias = "verifying")]
    Verifying,
    #[serde(alias = "installing")]
    Installing,
    #[serde(alias = "done")]
    Done,
}

#[derive(Serialize, Deserialize, Type, Debug)]
pub struct ProviderItem {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub app_type: AppType,
}

#[derive(Serialize, Deserialize, Debug, Type)]
pub struct LaunchOption {
    // The name of the option. May be an empty string when the option isn’t any special.
    pub description: String,
    // Path to executable that should be launched. Usually relative to working directory. However it can also be set as absolute
    pub executable: String,
    // Arguments that are to be provided to the executable
    pub arguments: String,
    // Absolute path to working directory from which the game has to be started.
    pub working_directory: String,
    // Array of Key - Value tuples. That describe additional environment variables that need to be set.
    pub environment: Vec<(String, String)>,
    // If known, this describes what the target actually is. Used for presentation to the user as well
    // as potentially for skipping game launchers if possible.
    pub launch_type: LaunchType,
    // List of tags that apply to this option. Tags point that the action is preferred when running
    // on particular piece of hardware. Currently defined hardware tags:
    //   'steamdeck': when on steam deck the launch option will be used instead of the default
    pub hardware_tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Type)]
pub struct InstallOptionDescription {
    // Identifier for the install option. This is used to provide
    // a common library-agnostic identifier for common install options.
    //   “language”: the language of the game to install
    //   “branch”: the branch of the game to install
    //   “version”: the version of the game to install
    //   “os”: the os platform of the game to install
    //   “architecture”: the cpu architecture of the game to install
    //   "verify": whether or not to verify the installation
    pub id: String,
    // The name of the provider-specific install option.
    // (E.g. “version”, “os”, “language”, “low-violence”)
    pub name: String,
    // A human-readable string of the install option appropriate for showing
    // in the UI. (E.g. “Version”, “Operating System”, etc.)
    pub human_readable_name: String,
    // Possible values that can be passed for this install option. For example,
    //  if this option was for the version of the game to install, this list
    // could be a list of versions of the game that are available to be installed
    // (e.g. [“v2.1”, “v2.0”, “v1.5”, “v1.3”])
    pub values: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Type, Value, OwnedValue, PartialEq, Copy, Clone)]
pub enum ProviderStatus {
    Unauthorized = 0,
    Requires2fa = 1,
    Authorized = 2,
}

impl ProviderStatus {
    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => Self::Unauthorized,
            1 => Self::Requires2fa,
            2 => Self::Authorized,
            _ => Self::Unauthorized,
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Type, Value, OwnedValue, PartialEq, Clone)]
pub struct ArtworkImage {
    pub url: String,
    pub image_type: String,
}

#[derive(Serialize, Deserialize, Debug, Type, Value, OwnedValue, PartialEq, Clone)]
pub struct ArtworkMetadata {
    item_id: String,
    provider: String,
    images: Vec<ArtworkImage>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub enum PlaytronAppType {
    #[default]
    Game,
    #[serde(alias = "DLC")]
    Dlc,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct PlaytronImage {
    #[serde(alias = "type")]
    pub image_type: String,
    pub url: String,
    #[serde(default)]
    pub alt: String,
    pub source: String,
}

#[derive(Serialize, Deserialize, Debug, Type, PartialEq, Clone)]
/// The current download stage of a downloading app from a plugin
pub enum DownloadStage {
    Preallocating = 0,
    Downloading = 1,
    Verifying = 2,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
/// This response is sent in the background while the download is in progress
pub struct AppDownloadProgress {
    /// Current stage of the donwload
    pub stage: DownloadStage,
    /// Progress, from 0.0 to 100.0
    pub progress: f32,
    /// Bytes already downloaded
    pub bytes: u64,
    /// Error if any happened
    pub error: String,
    /// Error code of the error
    pub error_code: Option<u16>,
    /// Last Modified header value from download request
    pub last_modified: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct PlaytronProvider {
    pub provider: String,
    #[serde(alias = "providerAppId")]
    pub provider_app_id: String,
    #[serde(alias = "storeId")]
    pub store_id: String,
    #[serde(alias = "parentStoreId")]
    pub parent_store_id: Option<String>,
    #[serde(alias = "lastImportedTimestamp", with = "optional_date_serializer")]
    pub last_imported_timestamp: Option<DateTime<Utc>>,
    #[serde(alias = "knownDlcStoreIds")]
    pub known_dlc_store_ids: Vec<String>,
    #[serde(alias = "namespace")]
    pub namespace: String,
    #[serde(alias = "productStoreLink", default)]
    pub product_store_link: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct PlaytronTag {
    pub tag: String,
    #[serde(alias = "type")]
    pub tag_type: String,
    pub source: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct ItemMetadata {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub providers: Vec<PlaytronProvider>,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<PlaytronTag>,
    #[serde(default)]
    pub images: Vec<PlaytronImage>,
    #[serde(default)]
    pub publishers: Vec<String>,
    #[serde(default)]
    pub developers: Vec<String>,
    #[serde(alias = "type")]
    pub app_type: PlaytronAppType,
    #[serde(default)]
    pub use_container_runtime: bool,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct RegistryEntry {
    pub language: Option<String>,
    pub group: String,
    pub key: String,
    pub value: String,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct Registry {
    pub dword: Option<Vec<RegistryEntry>>,
    pub string: Option<Vec<RegistryEntry>>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct RequirementOSConfig {
    pub is_64_bit_windows: Option<bool>,
    pub os_type: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct RunProcessParams {
    pub name: String,
    pub has_run_key: Option<String>,
    pub process: String,
    pub command: Option<String>,
    pub no_clean_up: Option<bool>,
    pub minimum_has_run_value: Option<String>,
    pub requirement_os: RequirementOSConfig,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct InstallScript {
    pub path: String,
    pub registry: Registry,
    pub run_process: Vec<RunProcessParams>,
}

#[derive(SerializeDict, Type, Deserialize)]
#[zvariant(signature = "a{sv}")]
pub struct InstallOption {
    pub language: Option<String>,
    pub os: Option<String>,
    pub verify: Option<bool>,
}

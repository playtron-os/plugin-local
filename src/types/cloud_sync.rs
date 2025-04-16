use serde::{Deserialize, Serialize};
use zbus::zvariant::Type;

#[derive(Debug, Serialize, Deserialize, PartialEq, Type)]
#[serde(rename_all = "lowercase")]
pub enum CloudSyncOperation {
    Download,
    Upload,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "failure_type", content = "details", rename_all = "lowercase")]
pub enum CloudFailureReason {
    /// No space on drive
    Disk { needed: u64 },
    /// No space in cloud
    Quota { total: u64, quota: u64 },
    /// Local and Remote timstamps
    Conflict { local: u64, remote: u64 },
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SaveSyncState {
    Progress,
    Failure,
    Success,
}

#[derive(Serialize, Deserialize)]
pub struct CloudSyncParams {
    pub user_id: String,
    pub provider_app_id: String,
    pub operation: CloudSyncOperation,
    /// If true the operation should be enforced
    pub conflict_resolution: bool,
}

#[derive(Serialize, Deserialize)]
pub struct SaveSyncProgress {
    pub owned_app_id: String,
    pub state: SaveSyncState,

    pub progress: Option<f32>,
    pub failure: Option<CloudFailureReason>,
}

#[derive(Serialize, Deserialize)]
pub struct AppSaveSyncRequest {
    pub owned_app_id: String,
    pub operation: CloudSyncOperation,
    #[serde(default)]
    pub conflict_resolution: bool,
}

// Plugin stuff

#[derive(Serialize, Deserialize, Debug, Type)]
pub struct CloudSyncProgress {
    pub app_id: String,
    pub progress: f64,
    pub sync_state: CloudSyncOperation,
}

#[derive(Serialize, Deserialize, Debug, Type)]
pub struct CloudSyncFailed {
    pub app_id: String,
    pub error: String,
    pub local: u64,
    pub remote: u64,
    pub usage: u64,
    pub quota: u64,
}

#[derive(Serialize, Deserialize, Debug, Type)]
pub struct CloudPath {
    pub alias: String,
    pub path: String,
    pub pattern: String,
    pub recursive: bool,
    pub platforms: Vec<String>,
}

use num_derive::FromPrimitive;
use thiserror::Error;

#[derive(Copy, Clone, Error, Debug, PartialEq, FromPrimitive)]
pub enum PluginError {
    #[error("one.playtron.Error.DiskNotFound")]
    DiskNotFound,
    #[error("one.playtron.Error.DownloadInProgress")]
    DownloadInProgress,
    #[error("one.playtron.Error.DownloadFailed")]
    DownloadFailed,
    #[error("one.playtron.Error.ContentNotFound")]
    ContentNotFound,
    #[error("one.playtron.Error.AppNotInstalled")]
    AppNotInstalled,
    #[error("one.playtron.Error.InvalidAppId")]
    InvalidAppId,
    #[error("one.playtron.Error.NotLoggedIn")]
    NotLoggedIn,
    #[error("one.playtron.Error.MissingDirectory")]
    MissingDirectory,
    #[error("one.playtron.Error.InvalidPassword")]
    InvalidPassword,
    #[error("one.playtron.Error.AuthenticationError")]
    AuthenticationError,
    #[error("one.playtron.Error.TfaTimedOut")]
    TfaTimedOut,
    #[error("one.playtron.Error.RateLimitExceeded")]
    RateLimitExceeded,
    #[error("one.playtron.Error.AppUpdateRequired")]
    AppUpdateRequired,
    #[error("one.playtron.Error.DependencyUpdateRequired")]
    DependencyUpdateRequired,
    #[error("one.playtron.Error.Timeout")]
    Timeout,
    #[error("one.playtron.Error.DependencyError")]
    DependencyError,
    #[error("one.playtron.Error.PreLaunchError")]
    PreLaunchError,
    #[error("one.playtron.Error.CloudConflict")]
    CloudConflict,
    #[error("one.playtron.Error.CloudQuota")]
    CloudQuota,
    #[error("one.playtron.Error.CloudFileDownload")]
    CloudFileDownload,
    #[error("one.playtron.Error.CloudFileUpload")]
    CloudFileUpload,
    #[error("one.playtron.Error.AppNotOwned")]
    AppNotOwned,
    #[error("one.playtron.Error.PlayingBlocked")]
    PlayingBlocked,
    #[error("one.playtron.Error.NotEnoughSpace")]
    NotEnoughSpace,
    #[error("one.playtron.Error.Permission")]
    Permission,
    #[error("one.playtron.Error.NetworkRequired")]
    NetworkRequired,
}

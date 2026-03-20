use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

use futures::StreamExt;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;
use tokio::time::sleep;
use zbus::Connection;

use crate::constants::{CLIENT_PATH, LIBRARY_SUBDIR};
use crate::local::connector::LocalConnector;
use crate::plugin::dbus::CONNECTION;
use crate::plugin::library_provider::LibraryProvider;

/// zbus proxy for subscribing to playserve's Manager drive signals.
#[zbus::proxy(
    interface = "one.playtron.plugin.Manager",
    default_service = "one.playtron.Playserve",
    default_path = "/one/playtron/plugins/Manager"
)]
trait PluginManager {
    #[zbus(signal)]
    fn on_drive_added(&self, drive: PluginDriveInfo) -> zbus::Result<()>;

    #[zbus(signal)]
    fn on_drive_removed(&self, drive_name: &str) -> zbus::Result<()>;
}

/// Mirrors playserve's PluginDriveInfo for deserialization of the D-Bus signal.
#[derive(Debug, Clone, serde::Deserialize, zbus::zvariant::Type)]
#[allow(dead_code)]
pub struct PluginDriveInfo {
    pub vendor: String,
    pub model: String,
    pub hint_name: String,
    pub name: String,
    pub available_space: u64,
    pub max_size: u64,
    pub path: String,
    pub file_system: String,
    pub is_root: bool,
    pub needs_formatting: bool,
}

/// Starts watching the local game directories for new or removed apps.
/// When changes are detected, emits the `installed_apps_updated` D-Bus signal
/// so playserve re-fetches the app list.
///
/// Listens to playserve's `on_drive_added` / `on_drive_removed` D-Bus signals
/// to detect external disk mount/unmount and update watches accordingly.
pub async fn start_watcher() {
    let connector = LocalConnector;

    // Ensure the home library path exists so we can watch it
    let home_library_path = match dirs::data_dir() {
        Some(data_dir) => data_dir.join(LIBRARY_SUBDIR),
        None => {
            log::error!("Failed to get data dir, cannot start watcher");
            return;
        }
    };
    if let Err(e) = std::fs::create_dir_all(&home_library_path) {
        log::error!("Failed to create local apps directory: {}", e);
        return;
    }

    // Channel: both filesystem events and drive events feed into this
    let (tx, mut rx) = mpsc::channel::<WatchEvent>(16);

    // Set up the filesystem watcher
    let fs_tx = tx.clone();
    let mut watcher = match create_fs_watcher(fs_tx) {
        Some(w) => w,
        None => return,
    };

    // Track watched paths so we can add/remove watches dynamically
    let mut watched_paths: HashSet<PathBuf> = HashSet::new();
    sync_watch_paths(
        &connector,
        &home_library_path,
        &mut watcher,
        &mut watched_paths,
    );

    // Subscribe to playserve drive signals in a background task
    let drive_tx = tx.clone();
    tokio::spawn(subscribe_to_drive_signals(drive_tx));

    // Take a snapshot of the current apps
    let mut known_apps = list_current_apps(&connector);

    log::info!("Filesystem watcher started");

    // Event loop
    loop {
        let Some(event) = rx.recv().await else {
            log::warn!("Watcher channel closed");
            break;
        };

        match event {
            WatchEvent::FilesystemChange => {
                // Debounce: wait and drain any additional fs events
                sleep(Duration::from_secs(2)).await;
                while rx.try_recv().is_ok() {}

                check_for_changes(&connector, &mut known_apps).await;
            }
            WatchEvent::DriveChanged => {
                // Give the system a moment to finish mounting/unmounting
                sleep(Duration::from_secs(3)).await;
                while rx.try_recv().is_ok() {}

                sync_watch_paths(
                    &connector,
                    &home_library_path,
                    &mut watcher,
                    &mut watched_paths,
                );
                check_for_changes(&connector, &mut known_apps).await;
            }
        }
    }
}

#[derive(Debug)]
enum WatchEvent {
    FilesystemChange,
    DriveChanged,
}

/// Subscribes to playserve's on_drive_added / on_drive_removed D-Bus signals.
async fn subscribe_to_drive_signals(tx: mpsc::Sender<WatchEvent>) {
    // Wait a bit for playserve to be available on the bus
    sleep(Duration::from_secs(5)).await;

    let conn = match Connection::session().await {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to connect to session bus for drive signals: {}", e);
            return;
        }
    };

    let proxy = match PluginManagerProxy::new(&conn).await {
        Ok(p) => p,
        Err(e) => {
            log::error!("Failed to create PluginManager proxy: {}", e);
            return;
        }
    };

    // Subscribe to both signals
    let mut drive_added = match proxy.receive_on_drive_added().await {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to subscribe to on_drive_added: {}", e);
            return;
        }
    };
    let mut drive_removed = match proxy.receive_on_drive_removed().await {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to subscribe to on_drive_removed: {}", e);
            return;
        }
    };

    log::info!("Subscribed to playserve drive D-Bus signals");

    loop {
        tokio::select! {
            Some(signal) = drive_added.next() => {
                if let Ok(args) = signal.args() {
                    log::info!("Drive added: {:?}", args.drive.name);
                }
                let _ = tx.send(WatchEvent::DriveChanged).await;
            }
            Some(signal) = drive_removed.next() => {
                if let Ok(args) = signal.args() {
                    log::info!("Drive removed: {}", args.drive_name);
                }
                let _ = tx.send(WatchEvent::DriveChanged).await;
            }
            else => {
                log::warn!("Drive signal streams ended");
                break;
            }
        }
    }
}

/// Re-scans library paths and adds/removes watches as needed.
/// Returns true if the set of watched paths changed.
fn sync_watch_paths(
    connector: &LocalConnector,
    home_library_path: &Path,
    watcher: &mut RecommendedWatcher,
    watched_paths: &mut HashSet<PathBuf>,
) -> bool {
    let mut current_paths: HashSet<PathBuf> = HashSet::new();
    current_paths.insert(home_library_path.to_path_buf());
    if let Ok(paths) = connector.get_library_paths() {
        current_paths.extend(paths);
    }

    let to_add: Vec<_> = current_paths.difference(watched_paths).cloned().collect();
    let to_remove: Vec<_> = watched_paths.difference(&current_paths).cloned().collect();

    if to_add.is_empty() && to_remove.is_empty() {
        return false;
    }

    for path in &to_remove {
        log::info!("Unwatching removed library path: {:?}", path);
        if let Err(e) = watcher.unwatch(path) {
            log::warn!("Failed to unwatch {:?}: {}", path, e);
        }
    }

    for path in &to_add {
        log::info!("Watching new library path: {:?}", path);
        if let Err(e) = watcher.watch(path, RecursiveMode::NonRecursive) {
            log::warn!("Failed to watch {:?}: {}", path, e);
        }
    }

    *watched_paths = current_paths;
    true
}

async fn check_for_changes(connector: &LocalConnector, known_apps: &mut HashSet<String>) {
    let current_apps = list_current_apps(connector);
    if current_apps == *known_apps {
        return;
    }

    let added: Vec<_> = current_apps.difference(known_apps).collect();
    let removed: Vec<_> = known_apps.difference(&current_apps).collect();
    log::info!(
        "Local apps changed - added: {:?}, removed: {:?}",
        added,
        removed
    );
    *known_apps = current_apps;

    if let Err(e) = emit_installed_apps_updated().await {
        log::error!("Failed to emit installed_apps_updated signal: {}", e);
    }
}

fn create_fs_watcher(tx: mpsc::Sender<WatchEvent>) -> Option<RecommendedWatcher> {
    match notify::recommended_watcher(move |res: Result<notify::Event, _>| {
        if let Ok(event) = res {
            use notify::EventKind::*;
            match event.kind {
                Create(_) | Remove(_) | Modify(_) => {
                    let _ = tx.blocking_send(WatchEvent::FilesystemChange);
                }
                _ => {}
            }
        }
    }) {
        Ok(w) => Some(w),
        Err(e) => {
            log::error!("Failed to create filesystem watcher: {}", e);
            None
        }
    }
}

fn list_current_apps(connector: &LocalConnector) -> HashSet<String> {
    let paths = match connector.get_library_paths() {
        Ok(p) => p,
        Err(e) => {
            log::warn!("Failed to get library paths: {}", e);
            return HashSet::new();
        }
    };
    let mut apps = HashSet::new();
    for path in paths {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let is_dir = entry.metadata().map(|m| m.is_dir()).unwrap_or(false);
                let is_symlink = entry.file_type().map(|t| t.is_symlink()).unwrap_or(false);
                if (is_dir || is_symlink) && entry.path().join("gameinfo.yaml").exists() {
                    if let Some(name) = entry.file_name().to_str() {
                        apps.insert(name.to_string());
                    }
                }
            }
        }
    }
    apps
}

async fn emit_installed_apps_updated() -> Result<(), Box<dyn std::error::Error>> {
    let conn_guard = CONNECTION.lock().await;
    let conn = conn_guard.as_ref().ok_or("No D-Bus connection")?;
    let iface_ref = conn
        .object_server()
        .interface::<_, LibraryProvider>(CLIENT_PATH)
        .await?;
    LibraryProvider::installed_apps_updated(iface_ref.signal_emitter()).await?;
    log::info!("Emitted installed_apps_updated D-Bus signal");
    Ok(())
}

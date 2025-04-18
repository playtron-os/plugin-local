/// This module is where you implement the functionality to interact with the store service
/// legendary / gog-warp / etc code should go here. The module can be renamed to represent your
/// connector more accurately eg `legendary.rs`
use crate::types::app::InstalledApp;
use crate::types::results::{EmptyResult, ResultWithError};
use crate::utils::disks::get_mount_points;
use dirs;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Clone)]
pub struct LocalConnector;

#[derive(Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    account: Option<String>,
}

impl LocalConnector {
    pub fn get_config_path(&self) -> PathBuf {
        dirs::data_dir()
            .unwrap()
            .join(PathBuf::from("playtron/plugins/local"))
    }

    pub fn load_auth(&self) -> Option<String> {
        let auth_file = self.get_config_path().join("auth.json");
        if fs::metadata(&auth_file).is_err() {
            return None;
        }
        let auth = fs::read_to_string(&auth_file).unwrap();
        if let Ok(account_info) = serde_json::from_str::<AccountInfo>(&auth) {
            account_info.account.clone()
        } else {
            None
        }
    }

    pub fn save_auth(&mut self, account: &str) -> EmptyResult {
        let account_info = AccountInfo {
            account: Some(account.to_string()),
        };
        match fs::write(
            self.get_config_path().join("auth.json"),
            serde_json::to_string_pretty(&account_info)?,
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn delete_auth(&mut self) -> EmptyResult {
        match fs::remove_file(self.get_config_path().join("auth.json")) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
    pub fn get_library_paths(&self) -> Vec<PathBuf> {
        let mut library_paths = Vec::new();
        let home_library_path = dirs::data_dir().unwrap().join("playtron/apps/local");
        if home_library_path.exists() {
            library_paths.push(home_library_path);
        }
        for mount_point in get_mount_points() {
            let library_path = PathBuf::from_str(&mount_point)
                .unwrap()
                .join("playtron/apps/local");
            if library_path.exists() {
                library_paths.push(library_path);
            }
        }
        library_paths
    }

    pub async fn list_apps(&self) -> ResultWithError<Vec<String>> {
        let mut app_list = Vec::new();
        for library_path in self.get_library_paths() {
            for entry in fs::read_dir(library_path)? {
                let path = entry?.path();
                if path.is_dir() || path.is_symlink() {
                    app_list.push(path.file_name().unwrap().to_str().unwrap().to_string())
                }
            }
        }
        Ok(app_list)
    }

    pub fn find_app(&self, app_id: &str) -> Option<PathBuf> {
        for library_path in self.get_library_paths() {
            for entry in fs::read_dir(&library_path).unwrap() {
                let path = entry.unwrap().path();
                let dir_name = path.file_name().unwrap().to_str().unwrap().to_string();
                if dir_name == app_id {
                    return Some(library_path.join(path));
                }
            }
        }
        None
    }

    pub async fn list_installed_apps(&self) -> ResultWithError<Vec<InstalledApp>> {
        let mut apps: Vec<InstalledApp> = vec![];
        for library_path in self.get_library_paths() {
            for entry in fs::read_dir(library_path)? {
                let app_id = entry?;
                let path = &app_id.path();
                let installed_app: InstalledApp = InstalledApp {
                    app_id: app_id.file_name().to_str().unwrap().to_string(),
                    installed_path: path.to_str().unwrap().to_string(),
                    downloaded_bytes: 1,
                    total_download_size: 1,
                    disk_size: 1,
                    version: "1.0".to_string(),
                    latest_version: "1.0".to_string(),
                    update_pending: false,
                    os: "windows".to_string(),
                    disabled_dlc: [].to_vec(),
                };
                apps.push(installed_app);
            }
        }
        Ok(apps)
    }

    pub fn write_installed_app(&self, app_id: &str, installed_app: &InstalledApp) -> EmptyResult {
        let config_path = self.get_config_path();
        let installed_apps_dir = config_path.join("apps");
        if !installed_apps_dir.exists() {
            fs::create_dir_all(&installed_apps_dir).expect("Failed to create directory");
        }
        let installed_app_path = installed_apps_dir.join(format!("{}.json", app_id));
        fs::write(
            installed_app_path,
            serde_json::to_string_pretty(&installed_app)?,
        )
        .expect("Failed to write file");
        Ok(())
    }

    pub async fn get_installed_app(&self, app_id: &str) -> ResultWithError<InstalledApp> {
        let config_path = self.get_config_path();
        let installed_apps_dir = config_path.join("apps");
        let installed_app_path = installed_apps_dir.join(format!("{}.json", app_id));
        let content = fs::read_to_string(installed_app_path)?;
        Ok(serde_json::from_str(&content).unwrap())
    }

    pub async fn load_metadata(&self, app_id: &str) -> ResultWithError<BTreeMap<String, String>> {
        let install_path = match self.find_app(app_id) {
            Some(install_path) => install_path,
            None => {
                return Err(format!("Couldn't find install path for {}", app_id).into());
            }
        };
        let metadata_path = install_path.join("gameinfo.yaml");
        let metadata: BTreeMap<String, String> =
            serde_yaml::from_str(&fs::read_to_string(metadata_path).unwrap()).unwrap();
        Ok(metadata)
    }

    pub async fn uninstall(&self, app_id: &str) -> ResultWithError<()> {
        let installed_app = match self.get_installed_app(app_id).await {
            Ok(app) => Some(app),
            Err(e) => {
                log::error!("{}", e.to_string());
                None
            }
        };
        if installed_app.is_none() {
            log::warn!(
                "Couldn't find app_id {} in metadata dir, skipping delete",
                app_id
            );
            return Ok(());
        }
        let installed_app = installed_app.unwrap();
        log::info!("Removing {}", installed_app.installed_path);
        match fs::remove_dir_all(&installed_app.installed_path) {
            Ok(_) => Ok(()),
            Err(e) => {
                Err(format!("Failed to remove {}: {}", installed_app.installed_path, e).into())
            }
        }
    }
}

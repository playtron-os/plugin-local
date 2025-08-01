/// This module is where you implement the functionality to interact with the store service
/// legendary / gog-warp / etc code should go here. The module can be renamed to represent your
/// connector more accurately eg `legendary.rs`
use crate::types::app::InstalledApp;
use crate::types::results::ResultWithError;
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
    pub fn get_library_paths(&self) -> ResultWithError<Vec<PathBuf>> {
        let mut library_paths = Vec::new();
        let _data_dir = dirs::data_dir().ok_or("Can't get data dir")?;
        let home_library_path = _data_dir.join("playtron/apps/local");
        if home_library_path.exists() {
            library_paths.push(home_library_path);
        }
        for mount_point in get_mount_points() {
            let library_path = PathBuf::from_str(&mount_point)?.join("playtron/apps/local");
            if library_path.exists() {
                library_paths.push(library_path);
            }
        }
        Ok(library_paths)
    }

    pub async fn list_apps(&self) -> ResultWithError<Vec<String>> {
        let mut app_list = Vec::new();
        for library_path in self.get_library_paths()? {
            for entry in fs::read_dir(library_path)? {
                let path = entry?.path();
                if path.is_dir() || path.is_symlink() {
                    app_list.push(
                        path.file_name()
                            .ok_or("Failed to read file name")?
                            .to_str()
                            .ok_or("Failed to convert path")?
                            .to_string(),
                    )
                }
            }
        }
        Ok(app_list)
    }

    pub fn find_app(&self, app_id: &str) -> ResultWithError<Option<PathBuf>> {
        for library_path in self.get_library_paths()? {
            for entry in fs::read_dir(&library_path)? {
                let path = entry?.path();
                let dir_name = path
                    .file_name()
                    .ok_or("Failed to read file name")?
                    .to_str()
                    .ok_or("Failed to convert path")?
                    .to_string();
                if dir_name == app_id {
                    return Ok(Some(library_path.join(path)));
                }
            }
        }
        Ok(None)
    }

    pub async fn list_installed_apps(&self) -> ResultWithError<Vec<InstalledApp>> {
        let mut apps: Vec<InstalledApp> = vec![];
        for library_path in self.get_library_paths()? {
            for entry in fs::read_dir(library_path)? {
                let dir_entry = entry?;
                if dir_entry
                    .metadata()
                    .is_ok_and(|metadata| metadata.is_file())
                {
                    continue;
                }

                match self.get_installed_app(&dir_entry).await {
                    Ok(installed_app) => apps.push(installed_app),
                    Err(err) => log::error!("Failed to get installed app data {err}"),
                }
            }
        }
        Ok(apps)
    }

    async fn get_installed_app(&self, dir_entry: &fs::DirEntry) -> ResultWithError<InstalledApp> {
        let app_id = dir_entry
            .file_name()
            .to_str()
            .ok_or("Can't get directory name")?
            .to_string();
        let metadata = self.load_metadata(&app_id).await?;
        let os: String = match metadata.get("os").and_then(|os| os.as_str()) {
            Some(platform) => platform.to_string(),
            None => "windows".to_string(),
        };
        let path = dir_entry.path();
        let disk_size = 1;

        Ok(InstalledApp {
            app_id,
            installed_path: path
                .to_str()
                .ok_or("Failed to read installed path")?
                .to_string(),
            downloaded_bytes: disk_size,
            total_download_size: disk_size,
            disk_size,
            version: "1.0".to_string(),
            latest_version: "1.0".to_string(),
            update_pending: false,
            os,
            language: "".to_string(),
            disabled_dlc: [].to_vec(),
        })
    }

    pub async fn load_metadata(
        &self,
        app_id: &str,
    ) -> ResultWithError<BTreeMap<String, serde_yaml::Value>> {
        let install_path = match self.find_app(app_id)? {
            Some(install_path) => install_path,
            None => {
                return Err(format!("Couldn't find install path for {}", app_id).into());
            }
        };
        let metadata_path = install_path.join("gameinfo.yaml");
        if !metadata_path.exists() {
            return Err(format!("Metadata file for {} doesn't exist", app_id).into());
        }
        let metadata: BTreeMap<String, serde_yaml::Value> =
            serde_yaml::from_str(&fs::read_to_string(metadata_path)?)?;
        Ok(metadata)
    }

    pub async fn uninstall(&self, app_id: &str) -> ResultWithError<()> {
        let install_path = match self.find_app(app_id)? {
            Some(install_path) => install_path,
            None => {
                return Err(format!("Couldn't find install path for {}", app_id).into());
            }
        };
        log::info!("Removing {:?}", &install_path);
        if install_path.exists() {
            match fs::remove_dir_all(&install_path) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Failed to remove {:?}: {}", install_path, e).into()),
            }
        } else {
            Ok(())
        }
    }
}

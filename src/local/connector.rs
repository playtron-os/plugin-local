/// This module is where you implement the functionality to interact with the store service
/// legendary / gog-warp / etc code should go here. The module can be renamed to represent your
/// connector more accurately eg `legendary.rs`
///
///
use crate::types::app::InstalledApp;
use crate::types::results::{EmptyResult, ResultWithError};
use dirs;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const BASE_URL: &str = "http://testprovider.lutris.org";

#[derive(Deserialize, Debug)]
struct AppList {
    apps: Vec<String>,
}

#[derive(Clone)]
pub struct ExampleConnector;

#[derive(Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    account: Option<String>,
}

impl ExampleConnector {
    pub fn get_config_path(&self) -> PathBuf {
        dirs::data_dir()
            .unwrap()
            .join(PathBuf::from("playtron/plugins/local"))
    }

    pub async fn list_apps(&self) -> ResultWithError<Vec<String>> {
        let apps_url = format!("{}/metadata/index.json", BASE_URL);
        let app_list = reqwest::get(&apps_url).await?.text().await?;
        match serde_json::from_str::<AppList>(&app_list) {
            Ok(index) => Ok(index.apps),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn list_installed_apps(&self) -> ResultWithError<Vec<InstalledApp>> {
        let config_path = self.get_config_path();
        let installed_apps_dir = config_path.join("apps");
        if !installed_apps_dir.exists() {
            return Ok(vec![]);
        }
        let mut apps: Vec<InstalledApp> = vec![];
        for entry in fs::read_dir(installed_apps_dir)? {
            let path = entry?.path();
            if path.is_file() && path.extension().unwrap_or_default() == "json" {
                let metadata_path = path.clone();
                let metadata: InstalledApp =
                    serde_json::from_str(&fs::read_to_string(metadata_path).unwrap()).unwrap();
                apps.push(metadata);
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
        let metadata_url = format!("{}/metadata/{}.yaml", BASE_URL, app_id);
        let metadata_content = match reqwest::get(&metadata_url).await?.text().await {
            Ok(s) => s,
            Err(e) => {
                log::error!("{}", e.to_string());
                return Err(format!("Failed to read metadata for {}", metadata_url).into());
            }
        };
        let metadata: BTreeMap<String, String> = serde_yaml::from_str(&metadata_content).unwrap();
        Ok(metadata)
    }

    pub fn get_image(&self, app_id: &str, format: &str) -> ResultWithError<String> {
        Ok(format!("{}/images/{}/{}.jpg", BASE_URL, format, app_id))
    }

    pub fn get_download_url(&self, file_name: &str) -> ResultWithError<String> {
        Ok(format!("{}/downloads/{}", BASE_URL, file_name))
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

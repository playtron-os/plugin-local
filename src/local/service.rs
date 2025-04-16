use crate::constants::LIBRARY_PROVIDER_ID;

use crate::plugin::library_provider::LibraryProviderSignals;
use crate::types::app::{
    self, DownloadStage, EulaEntry, InstalledApp, ItemMetadata, LaunchOption, PlaytronImage,
    PlaytronProvider, ProviderItem,
};
use crate::types::cloud_sync::CloudPath;
use crate::types::results::ResultWithError;
use futures::future;
use futures_util::StreamExt;
use std::cmp::min;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::vec;
use zbus::fdo;
use zbus::object_server::SignalEmitter;

use super::connector::ExampleConnector;

#[derive(Clone)]
pub struct ExampleService {
    connector: ExampleConnector,
}

impl ExampleService {
    pub fn new() -> Self {
        let connector = ExampleConnector {};
        Self { connector }
    }

    pub async fn install(
        &self,
        app_id: &str,
        base_path: String,
        options: HashMap<String, zbus::zvariant::Value<'_>>,
        emitter: SignalEmitter<'_>,
    ) -> ResultWithError<i32> {
        log::info!("Install {} to {}", app_id, base_path);
        let metadata = self.connector.load_metadata(app_id).await.unwrap();
        let path = format!("{}/{}", base_path, app_id);
        let download_size = metadata
            .get("download_size")
            .unwrap()
            .parse::<u64>()
            .unwrap();

        let platform: String = match options.get("platform") {
            Some(platform) => platform.to_string(),
            None => "windows".to_string(),
        };

        let installed_app = InstalledApp {
            app_id: metadata.get("id").unwrap().to_string(),
            installed_path: path.clone(),
            downloaded_bytes: 0,
            total_download_size: download_size,
            disk_size: metadata.get("disk_size").unwrap().parse::<u64>().unwrap(),
            version: metadata.get("version").unwrap().to_string(),
            latest_version: metadata.get("version").unwrap().to_string(),
            update_pending: false,
            os: platform.clone(),
            disabled_dlc: Vec::new(),
        };
        self.connector
            .write_installed_app(app_id, &installed_app)
            .unwrap();

        log::info!("Install {} to {}", app_id, base_path);
        let app_id = app_id.to_owned();
        let emitter = emitter.into_owned();
        let file_name = metadata.get("file_name").unwrap().to_owned();
        let url = self.connector.get_download_url(&file_name).unwrap();

        tokio::spawn(async move {
            LibraryProviderSignals::install_started(
                &emitter,
                installed_app.app_id.to_string(),
                installed_app.version.to_string(),
                installed_app.installed_path.to_string(),
                installed_app.total_download_size,
                false,
                installed_app.os,
            )
            .await
            .unwrap();
            let client = reqwest::Client::new();
            let res = client
                .get(&url)
                .send()
                .await
                .or(Err(format!("Failed to GET from '{}'", &url)))
                .unwrap();
            let total_size = res
                .content_length()
                .ok_or(format!("Failed to get content length from '{}'", &url))
                .unwrap();

            let mut downloaded: u64 = 0;
            let installed_dir = PathBuf::from(&path);
            if !installed_dir.exists() {
                std::fs::create_dir_all(&installed_dir).unwrap();
            }
            let archive_path = format!("{}/{}", path, file_name);
            let mut dest_file = File::create(&archive_path)
                .or(Err(format!("Failed to create file '{}'", path)))
                .unwrap();

            let mut stream = res.bytes_stream();
            while let Some(item) = stream.next().await {
                let chunk = match item {
                    Ok(chunk) => chunk,
                    Err(e) => {
                        log::error!("{}", e);
                        LibraryProviderSignals::install_failed(
                            &emitter,
                            &app_id,
                            "Unexpected server response",
                        )
                        .await
                        .unwrap();
                        return;
                    }
                };
                dest_file
                    .write_all(&chunk)
                    .or(Err("Error while writing to file".to_string()))
                    .unwrap();
                let new = min(downloaded + (chunk.len() as u64), total_size);
                downloaded = new;
                let progress = (downloaded as f64 / total_size as f64) * 100.0;
                LibraryProviderSignals::install_progressed(
                    &emitter,
                    app_id.to_owned(),
                    DownloadStage::Downloading,
                    downloaded,
                    total_size,
                    progress,
                )
                .await
                .unwrap();
            }
            let target = PathBuf::from(&path);
            let archive_file = File::open(&archive_path)
                .or(Err("Failed to open file "))
                .unwrap();

            zip_extract::extract(archive_file, &target, true).unwrap();
            let source_path = PathBuf::from(archive_path);
            std::fs::remove_file(&source_path).ok();

            LibraryProviderSignals::install_completed(&emitter.clone(), app_id.to_owned())
                .await
                .unwrap();
        });

        Ok(0)
    }

    pub async fn _get_provider_item(&self, app_id: &str) -> ProviderItem {
        let metadata = self.connector.load_metadata(app_id).await.unwrap();
        ProviderItem {
            id: metadata.get("id").unwrap().to_string(),
            name: metadata.get("name").unwrap().to_string(),
            provider: LIBRARY_PROVIDER_ID.to_string(),
            app_type: crate::types::app::AppType::Game,
        }
    }

    pub async fn get_provider_item(&self, app_id: &str) -> fdo::Result<ProviderItem> {
        Ok(self._get_provider_item(app_id).await)
    }

    pub async fn get_provider_items(&self) -> fdo::Result<Vec<ProviderItem>> {
        Ok(future::join_all(
            self.connector
                .list_apps()
                .await
                .unwrap()
                .into_iter()
                .map(|app_id| async move { self._get_provider_item(&app_id.clone()).await }),
        )
        .await)
    }

    pub async fn get_item_metadata(&self, app_id: &str) -> String {
        let metadata = self.connector.load_metadata(app_id).await.unwrap();
        let item_meta = ItemMetadata {
            id: app_id.to_owned(),
            name: metadata.get("name").unwrap().to_owned(),
            app_type: crate::types::app::PlaytronAppType::Game,
            providers: vec![PlaytronProvider {
                namespace: LIBRARY_PROVIDER_ID.to_string(),
                provider: LIBRARY_PROVIDER_ID.to_string(),
                provider_app_id: app_id.to_owned(),
                store_id: app_id.to_owned(),
                product_store_link: metadata.get("website").unwrap().to_owned(),
                parent_store_id: None,
                last_imported_timestamp: None,
                known_dlc_store_ids: vec![],
            }],
            summary: metadata.get("description").unwrap().to_owned(),
            description: metadata.get("description").unwrap().to_owned(),
            slug: app_id.to_owned(),
            developers: vec![metadata.get("description").unwrap().to_owned()],
            publishers: vec![],
            tags: vec![],
            images: vec![
                PlaytronImage {
                    image_type: "OfferImageTall".to_string(),
                    url: self
                        .connector
                        .get_image(app_id, "portrait")
                        .unwrap()
                        .to_owned(),
                    source: LIBRARY_PROVIDER_ID.to_string(),
                    alt: "".to_string(),
                },
                PlaytronImage {
                    image_type: "header".to_string(),
                    url: self
                        .connector
                        .get_image(app_id, "landscape")
                        .unwrap()
                        .to_owned(),
                    source: "steam".to_string(),
                    alt: "".to_string(),
                },
            ],
        };
        serde_json::to_string(&item_meta).unwrap()
    }

    pub async fn get_installed_apps(&self) -> ResultWithError<Vec<InstalledApp>> {
        self.connector.list_installed_apps().await
    }

    pub async fn get_post_install_steps(&self, app_id: &str) -> fdo::Result<String> {
        log::info!("Get post install steps for {}", app_id);
        Ok("[]".to_string())
    }

    pub async fn get_eulas(
        &self,
        app_id: &str,
        country: &str,
        locale: &str,
    ) -> fdo::Result<Vec<EulaEntry>> {
        log::info!(
            "Get eulas for {} (Country: {}, locale: {})",
            app_id,
            country,
            locale
        );
        Ok(vec![])
    }

    pub async fn pre_launch_hook(
        &self,
        app_id: String,
        using_offline_mode: bool,
    ) -> fdo::Result<Vec<String>> {
        log::info!(
            "pre launch hook for app_id {} (offline mode: {})",
            &app_id,
            using_offline_mode
        );

        Ok(vec![])
    }

    pub async fn get_launch_options(&self, app_id: &str) -> fdo::Result<Vec<LaunchOption>> {
        log::info!("get launch options for {}", app_id);
        let metadata = self.connector.load_metadata(app_id).await.unwrap();
        Ok(vec![LaunchOption {
            description: "Launch".to_string(),
            executable: metadata.get("executable").unwrap().to_owned(),
            working_directory: "".to_string(),
            environment: vec![],
            arguments: "".to_string(),
            hardware_tags: vec![],
            launch_type: app::LaunchType::Game,
        }])
    }

    pub async fn move_item(&self, app_id: &str, dest_path: &str) -> fdo::Result<()> {
        log::info!("Move {} to {}", app_id, dest_path);
        Ok(())
    }

    pub async fn uninstall(&self, app_id: &str) -> fdo::Result<()> {
        log::info!("Uninstall {}", app_id);
        match self.connector.uninstall(app_id).await {
            Ok(_) => Ok(()),
            Err(e) => Err(fdo::Error::Failed(format!("{}", e))),
        }
    }

    pub async fn get_save_path_patterns(
        &self,
        app_id: &str,
        platform: &str,
    ) -> fdo::Result<Vec<CloudPath>> {
        log::info!(
            "Get save path patterns {} for platform {}",
            app_id,
            platform
        );
        Ok(Vec::new())
    }

    pub async fn import(&self, app_id: &str, install_folder: &str) -> fdo::Result<()> {
        log::info!("Import {} from {}", app_id, install_folder);
        Ok(())
    }
}

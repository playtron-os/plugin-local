use crate::constants::LIBRARY_PROVIDER_ID;
use crate::types::app::{
    self, EulaEntry, InstalledApp, ItemMetadata, LaunchOption, PlaytronImage, PlaytronProvider,
    ProviderItem,
};
use crate::types::cloud_sync::CloudPath;
use crate::types::results::ResultWithError;
use futures::future;
use rsa::pkcs1::EncodeRsaPublicKey;
use rsa::pkcs8::LineEnding;
use rsa::{RsaPrivateKey, RsaPublicKey};
use std::collections::BTreeMap;
use std::vec;
use zbus::fdo;

use super::connector::LocalConnector;

#[derive(Clone)]
pub struct LocalService {
    rsa: RsaPrivateKey,
    connector: LocalConnector,
}

impl LocalService {
    pub fn new() -> Self {
        let connector = LocalConnector {};
        let mut rng = rand::thread_rng();
        let rsa = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate new key");
        Self { rsa, connector }
    }

    pub fn get_public_key(&self) -> String {
        let public_key = RsaPublicKey::from(&self.rsa);
        public_key
            .to_pkcs1_pem(LineEnding::LF)
            .map(|pem| pem.to_string())
            .unwrap_or_else(|e| {
                log::warn!("Failed to convert public key to PEM: {}", e);
                String::new()
            })
    }

    pub async fn _get_provider_item(&self, app_id: &str) -> ResultWithError<ProviderItem> {
        let metadata = self.connector.load_metadata(app_id).await?;
        Ok(ProviderItem {
            id: app_id.to_string(),
            name: metadata
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or(app_id)
                .to_string(),
            provider: LIBRARY_PROVIDER_ID.to_string(),
            app_type: crate::types::app::AppType::Game,
        })
    }

    pub async fn get_provider_item(&self, app_id: &str) -> ResultWithError<ProviderItem> {
        self._get_provider_item(app_id).await
    }

    pub async fn get_provider_items(&self) -> ResultWithError<Vec<ProviderItem>> {
        let results = future::join_all(
            self.connector
                .list_apps()
                .await?
                .into_iter()
                .map(|app_id| async move { self._get_provider_item(&app_id.clone()).await }),
        )
        .await;

        Ok(results.into_iter().filter_map(Result::ok).collect())
    }

    pub fn get_images(&self, metadata: BTreeMap<String, serde_yaml::Value>) -> Vec<PlaytronImage> {
        let mut images = Vec::new();
        if let Some(image_url) = metadata.get("image").and_then(|img| img.as_str()) {
            images.push(PlaytronImage {
                image_type: "landscape".to_string(),
                url: image_url.to_owned(),
                source: "local".to_string(),
                alt: "".to_string(),
            })
        }
        images
    }

    pub async fn get_item_metadata(&self, app_id: &str) -> ResultWithError<String> {
        let metadata = self.connector.load_metadata(app_id).await?;
        let item_meta = ItemMetadata {
            id: app_id.to_owned(),
            name: metadata
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or(app_id)
                .to_owned(),
            app_type: crate::types::app::PlaytronAppType::Game,
            providers: vec![PlaytronProvider {
                namespace: LIBRARY_PROVIDER_ID.to_string(),
                provider: LIBRARY_PROVIDER_ID.to_string(),
                provider_app_id: app_id.to_owned(),
                store_id: app_id.to_owned(),
                product_store_link: "".to_string(),
                parent_store_id: None,
                last_imported_timestamp: None,
                known_dlc_store_ids: vec![],
            }],
            summary: "".to_string(),
            description: "".to_string(),
            slug: app_id.to_owned(),
            developers: vec![],
            publishers: vec![],
            tags: vec![],
            use_container_runtime: metadata
                .get("runtime")
                .and_then(|r| r.as_bool())
                .is_none_or(|r| r),
            images: self.get_images(metadata),
        };
        Ok(serde_json::to_string(&item_meta)?)
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

    pub async fn get_launch_options(&self, app_id: &str) -> ResultWithError<Vec<LaunchOption>> {
        log::info!("get launch options for {}", app_id);
        let metadata = self.connector.load_metadata(app_id).await?;
        if let Some(executable) = metadata.get("executable").and_then(|exe| exe.as_str()) {
            Ok(vec![LaunchOption {
                description: "Launch".to_string(),
                executable: executable.to_owned(),
                working_directory: "".to_string(),
                environment: vec![],
                arguments: "".to_string(),
                hardware_tags: vec![],
                launch_type: app::LaunchType::Game,
            }])
        } else {
            Ok(vec![])
        }
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

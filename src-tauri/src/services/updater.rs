use crate::utils::AppError;
use serde::{Deserialize, Serialize};
use semver::Version;
use reqwest::Client;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStatus {
    pub current_version: String,
    pub latest_version: Option<String>,
    pub update_available: bool,
    pub last_check: Option<i64>,
    pub download_url: Option<String>,
    pub release_notes: Option<String>,
    pub checking: bool,
}

pub struct UpdateService {
    client: Client,
    current_version: Version,
    update_url: String,
}

impl UpdateService {
    pub fn new(current_version: &str) -> Result<Self, AppError> {
        let version = Version::parse(current_version).map_err(|e| AppError {
            message: format!("Invalid version format: {}", e),
        })?;

        Ok(Self {
            client: Client::new(),
            current_version: version,
            update_url: "https://api.github.com/repos/your-repo/releases/latest".to_string(),
        })
    }

    pub async fn check_for_updates(&self) -> Result<UpdateStatus, AppError> {
        let response = self
            .client
            .get(&self.update_url)
            .header("User-Agent", "GLaunch")
            .send()
            .await
            .map_err(|e| AppError {
                message: format!("Failed to check for updates: {}", e),
            })?;

        let release: serde_json::Value = response.json().await.map_err(|e| AppError {
            message: format!("Failed to parse update response: {}", e),
        })?;

        let latest_version = release["tag_name"]
            .as_str()
            .and_then(|v| Version::parse(v.trim_start_matches('v')).ok());

        let update_available = if let Some(ref latest) = latest_version {
            latest > &self.current_version
        } else {
            false
        };

        Ok(UpdateStatus {
            current_version: self.current_version.to_string(),
            latest_version: latest_version.map(|v| v.to_string()),
            update_available,
            last_check: Some(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
            ),
            download_url: release["assets"][0]["browser_download_url"]
                .as_str()
                .map(String::from),
            release_notes: release["body"].as_str().map(String::from),
            checking: false,
        })
    }

    pub async fn download_update(&self) -> Result<(), AppError> {
        // Cette fonction serait implémentée pour utiliser tauri::updater
        Ok(())
    }
}
use serde::{Deserialize, Serialize};
use keyring::Entry;
use std::env;
use crate::utils::AppError;
use crate::log_debug;

#[derive(Debug, Clone)]
pub struct CompiledSecrets {
    pub igdb_client_id: String,
    pub igdb_client_secret: String,
    pub steam_api_key: String,
    pub epic_client_id: String,
    pub epic_client_secret: String,
    pub battlenet_client_id: String,
    pub battlenet_client_secret: String,
}

impl CompiledSecrets {
    pub fn new() -> Self {
        let is_dev = std::env::var("TAURI_ENV").unwrap_or_default() == "dev";

        if is_dev {
            log_debug!("Running in development mode, using .env file");
            dotenv::dotenv().ok();
        }

        Self {
            igdb_client_id: std::env::var("IGDB_CLIENT_ID").unwrap_or_default(),
            igdb_client_secret: std::env::var("IGDB_CLIENT_SECRET").unwrap_or_default(),
            steam_api_key: std::env::var("STEAM_API_KEY").unwrap_or_default(),
            epic_client_id: std::env::var("EPIC_CLIENT_ID").unwrap_or_default(),
            epic_client_secret: std::env::var("EPIC_CLIENT_SECRET").unwrap_or_default(),
            battlenet_client_id: std::env::var("BATTLENET_CLIENT_ID").unwrap_or_default(),
            battlenet_client_secret: std::env::var("BATTLENET_CLIENT_SECRET").unwrap_or_default(),
        }
    }
}

pub struct SecretsManager {
    app_name: String,
    secrets: CompiledSecrets,
}

impl SecretsManager {
    pub fn new(app_name: &str) -> Self {
        let is_dev = env::var("TAURI_ENV").unwrap_or_default() == "dev";

        if is_dev {
            log_debug!("Running in development mode, using .env file");
            dotenv::dotenv().ok();
        }

        Self {
            app_name: app_name.to_string(),
            secrets: CompiledSecrets::new(),
        }
    }

    pub fn get_compiled_secret(&self, key: &str) -> Result<String, AppError> {
        match key {
            "IGDB_CLIENT_ID" => Ok(self.secrets.igdb_client_id.clone()),
            "IGDB_CLIENT_SECRET" => Ok(self.secrets.igdb_client_secret.clone()),
            "STEAM_API_KEY" => Ok(self.secrets.steam_api_key.clone()),
            "EPIC_CLIENT_ID" => Ok(self.secrets.epic_client_id.clone()),
            "EPIC_CLIENT_SECRET" => Ok(self.secrets.epic_client_secret.clone()),
            "BATTLENET_CLIENT_ID" => Ok(self.secrets.battlenet_client_id.clone()),
            "BATTLENET_CLIENT_SECRET" => Ok(self.secrets.battlenet_client_secret.clone()),
            _ => Err(AppError { message: format!("Unknown compiled secret: {}", key) }),
        }
    }

    pub fn store_token(&self, service: &str, token: &str) -> Result<(), AppError> {
        let entry = Entry::new(&self.app_name, service);
        match entry.set_password(token) {
            Ok(_) => Ok(()),
            Err(e) => Err(AppError {
                message: format!("Failed to store token: {}", e)
            })
        }
    }

    pub fn get_token(&self, service: &str) -> Result<String, AppError> {
        let entry = Entry::new(&self.app_name, service);
        match entry.get_password() {
            Ok(pwd) => Ok(pwd),
            Err(e) => Err(AppError {
                message: format!("Failed to retrieve token: {}", e)
            })
        }
    }

    pub fn delete_token(&self, service: &str) -> Result<(), AppError> {
        let entry = Entry::new(&self.app_name, service);
        match entry.delete_password() {
            Ok(_) => Ok(()),
            Err(e) => Err(AppError {
                message: format!("Failed to delete token: {}", e)
            })
        }
    }

    pub fn validate_required_secrets(&self) -> Result<(), AppError> {
        let required_secrets = [
            "IGDB_CLIENT_ID",
            "IGDB_CLIENT_SECRET",
        ];

        for secret in required_secrets.iter() {
            if self.get_compiled_secret(secret)?.is_empty() {
                return Err(AppError { message: format!("Required secret {} is missing", secret) });
            }
        }

        Ok(())
    }
}
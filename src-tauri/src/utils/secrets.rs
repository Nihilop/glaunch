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
    pub epic_client_id_secret: String,
    pub battlenet_client_id: String,
    pub battlenet_client_secret: String,
}

impl CompiledSecrets {
    pub fn new() -> Result<Self, AppError> {
        let is_dev = env::var("TAURI_ENV").unwrap_or_default() == "dev";
        log_debug!("Running in {} mode", if is_dev { "development" } else { "production" });


        log_debug!("Loading IGDB_CLIENT_ID...");
        let igdb_client_id = env::var("IGDB_CLIENT_ID").map_err(|_| AppError { message: "IGDB_CLIENT_ID not set".to_string() })?;
        log_debug!("Loading IGDB_CLIENT_SECRET...");
        let igdb_client_secret = env::var("IGDB_CLIENT_SECRET").map_err(|_| AppError { message: "IGDB_CLIENT_SECRET not set".to_string() })?;
        log_debug!("Loading STEAM_API_KEY...");
        let steam_api_key = env::var("STEAM_API_KEY").map_err(|_| AppError { message: "STEAM_API_KEY not set".to_string() })?;
        log_debug!("Loading EPIC_CLIENT_ID...");
        let epic_client_id = env::var("EPIC_CLIENT_ID").map_err(|_| AppError { message: "EPIC_CLIENT_ID not set".to_string() })?;
        log_debug!("Loading EPIC_CLIENT_ID_SECRET...");
        let epic_client_id_secret = env::var("EPIC_CLIENT_ID_SECRET").map_err(|_| AppError { message: "EPIC_CLIENT_ID_SECRET not set".to_string() })?;
        log_debug!("Loading BATTLENET_CLIENT_ID...");
        let battlenet_client_id = env::var("BATTLENET_CLIENT_ID").map_err(|_| AppError { message: "BATTLENET_CLIENT_ID not set".to_string() })?;
        log_debug!("Loading BATTLENET_CLIENT_SECRET...");
        let battlenet_client_secret = env::var("BATTLENET_CLIENT_SECRET").map_err(|_| AppError { message: "BATTLENET_CLIENT_SECRET not set".to_string() })?;

        let secrets = Self {
          igdb_client_id,
          igdb_client_secret,
          steam_api_key,
          epic_client_id,
          epic_client_id_secret,
          battlenet_client_id,
          battlenet_client_secret,
        };

        log_debug!("IGDB Client ID present: {}", !secrets.igdb_client_id.is_empty());
        Ok(secrets)
    }
}

pub struct SecretsManager {
    app_name: String,
    secrets: CompiledSecrets,
}

impl SecretsManager {
    pub fn new(app_name: &str) -> Self {
        Self {
            app_name: app_name.to_string(),
            secrets: CompiledSecrets::new().expect("Failed to load secrets"),
        }
    }

    pub fn get_compiled_secret(&self, key: &str) -> Result<String, AppError> {
        match key {
            "IGDB_CLIENT_ID" => Ok(self.secrets.igdb_client_id.clone()),
            "IGDB_CLIENT_SECRET" => Ok(self.secrets.igdb_client_secret.clone()),
            "STEAM_API_KEY" => Ok(self.secrets.steam_api_key.clone()),
            "EPIC_CLIENT_ID" => Ok(self.secrets.epic_client_id.clone()),
            "EPIC_CLIENT_ID_SECRET" => Ok(self.secrets.epic_client_id_secret.clone()),
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

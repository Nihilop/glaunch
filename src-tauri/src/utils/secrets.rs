use serde::{Deserialize, Serialize};
use keyring::Entry;
use std::env;
use std::collections::HashMap;
use crate::utils::AppError;
use crate::log_debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    secrets: HashMap<String, String>,
}

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

        let config: Config = if !is_dev {
            // En production, lire le fichier de config embarqué
            log_debug!("Loading production config...");
            let config_str = include_str!("../../config.production.json");
            serde_json::from_str(config_str).map_err(|e| AppError {
                message: format!("Failed to parse config: {}", e)
            })?
        } else {
            // En dev, lire les variables d'environnement
            log_debug!("Loading development environment variables...");
            dotenv::dotenv().ok();
            let mut secrets = HashMap::new();
            for key in &[
                "IGDB_CLIENT_ID",
                "IGDB_CLIENT_SECRET",
                "STEAM_API_KEY",
                "EPIC_CLIENT_ID",
                "EPIC_CLIENT_ID_SECRET",
                "BATTLENET_CLIENT_ID",
                "BATTLENET_CLIENT_SECRET",
            ] {
                secrets.insert(
                    key.to_string(),
                    env::var(key).unwrap_or_default()
                );
            }
            Config { secrets }
        };

        let get_secret = |key: &str| {
            config.secrets.get(key).cloned().unwrap_or_default()
        };

        let secrets = Self {
            igdb_client_id: get_secret("IGDB_CLIENT_ID"),
            igdb_client_secret: get_secret("IGDB_CLIENT_SECRET"),
            steam_api_key: get_secret("STEAM_API_KEY"),
            epic_client_id: get_secret("EPIC_CLIENT_ID"),
            epic_client_id_secret: get_secret("EPIC_CLIENT_ID_SECRET"),
            battlenet_client_id: get_secret("BATTLENET_CLIENT_ID"),
            battlenet_client_secret: get_secret("BATTLENET_CLIENT_SECRET"),
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

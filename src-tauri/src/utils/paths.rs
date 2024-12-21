use tauri::Manager;
use std::path::PathBuf;
use tauri::AppHandle;
use crate::utils::AppError;
use std::fs;

pub struct AppPaths {
    pub root: PathBuf,
    pub media: PathBuf,
    pub database: PathBuf,
    pub logs: PathBuf,
    pub settings: PathBuf,
}

impl AppPaths {
    pub fn new(app: &AppHandle) -> Result<Self, AppError> {
        // Obtenir le dossier racine AppData
        let root = app.path()
            .app_data_dir()
            .map_err(|e| AppError {
                message: format!("Could not determine AppData directory: {}", e)
            })?;

        // Définir les sous-chemins
        let paths = Self {
            media: root.join("media"),
            database: root.join("data"),
            logs: root.join("logs"),
            settings: root.join("config"),
            root,
        };

        // Créer tous les dossiers nécessaires
        fs::create_dir_all(&paths.media).map_err(|e| AppError {
            message: format!("Failed to create media directory: {}", e)
        })?;
        fs::create_dir_all(&paths.database).map_err(|e| AppError {
            message: format!("Failed to create database directory: {}", e)
        })?;
        fs::create_dir_all(&paths.logs).map_err(|e| AppError {
            message: format!("Failed to create logs directory: {}", e)
        })?;
        fs::create_dir_all(&paths.settings).map_err(|e| AppError {
            message: format!("Failed to create settings directory: {}", e)
        })?;

        Ok(paths)
    }

    pub fn get_database_path(&self) -> PathBuf {
        self.database.join("games.db")
    }

    pub fn get_settings_path(&self) -> PathBuf {
        self.settings.join("settings.json")
    }

    pub fn get_log_path(&self) -> PathBuf {
        self.logs.join("app.log")
    }
}
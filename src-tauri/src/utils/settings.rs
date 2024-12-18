use crate::utils::AppError;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub start_with_windows: bool,
    pub minimize_to_tray: bool,
    pub check_updates_on_startup: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            start_with_windows: false,
            minimize_to_tray: true,
            check_updates_on_startup: true,
        }
    }
}

pub struct SettingsManager {
    settings_path: PathBuf,
    settings: AppSettings,
}

impl SettingsManager {
    pub fn new() -> Result<Self, AppError> {
        let app_data = env::var("APPDATA").map_err(|e| AppError {
            message: format!("Failed to get APPDATA path: {}", e),
        })?;
        let settings_dir = PathBuf::from(app_data).join("glaunch");
        let settings_path = settings_dir.join("settings.json");

        fs::create_dir_all(&settings_dir).map_err(|e| AppError {
            message: format!("Failed to create settings directory: {}", e),
        })?;

        let settings = if settings_path.exists() {
            let content = fs::read_to_string(&settings_path).map_err(|e| AppError {
                message: format!("Failed to read settings file: {}", e),
            })?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            AppSettings::default()
        };

        Ok(Self {
            settings_path,
            settings,
        })
    }

    pub fn save(&self) -> Result<(), AppError> {
        let content = serde_json::to_string_pretty(&self.settings).map_err(|e| AppError {
            message: format!("Failed to serialize settings: {}", e),
        })?;
        fs::write(&self.settings_path, content).map_err(|e| AppError {
            message: format!("Failed to save settings: {}", e),
        })?;
        Ok(())
    }

    pub fn get_settings(&self) -> &AppSettings {
        &self.settings
    }

    pub fn update_settings(&mut self, settings: AppSettings) -> Result<(), AppError> {
        self.settings = settings;
        self.save()?;
        self.update_autostart()?;
        Ok(())
    }

    fn update_autostart(&self) -> Result<(), AppError> {
        let startup_dir = dirs::home_dir()
            .ok_or_else(|| AppError {
                message: "Could not find home directory".to_string(),
            })?
            .join("AppData")
            .join("Roaming")
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
            .join("Startup");

        let shortcut_path = startup_dir.join("GLaunch.lnk");
        let current_exe = env::current_exe().map_err(|e| AppError {
            message: format!("Failed to get current exe path: {}", e),
        })?;

        if self.settings.start_with_windows {
            // Ici, vous devriez implémenter la création du raccourci Windows
            // Pour cet exemple, on simule juste la création
            Ok(())
        } else {
            if shortcut_path.exists() {
                fs::remove_file(shortcut_path).map_err(|e| AppError {
                    message: format!("Failed to remove startup shortcut: {}", e),
                })?;
            }
            Ok(())
        }
    }

    pub fn export_database(&self, path: PathBuf) -> Result<(), AppError> {
        let app_data = env::var("APPDATA").map_err(|e| AppError {
            message: format!("Failed to get APPDATA path: {}", e),
        })?;
        let db_path = PathBuf::from(app_data).join("glaunch").join("games.db");

        if !db_path.exists() {
            return Err(AppError {
                message: "Database file not found".to_string(),
            });
        }

        fs::copy(db_path, path).map_err(|e| AppError {
            message: format!("Failed to export database: {}", e),
        })?;
        Ok(())
    }

    pub fn import_database(&self, path: PathBuf) -> Result<(), AppError> {
        let app_data = env::var("APPDATA").map_err(|e| AppError {
            message: format!("Failed to get APPDATA path: {}", e),
        })?;
        let db_dir = PathBuf::from(app_data).join("glaunch");
        let db_path = db_dir.join("games.db");

        // Créer une sauvegarde
        if db_path.exists() {
            let backup_path = db_dir.join("games.db.backup");
            fs::copy(&db_path, &backup_path).map_err(|e| AppError {
                message: format!("Failed to create database backup: {}", e),
            })?;
        }

        fs::copy(path, db_path).map_err(|e| AppError {
            message: format!("Failed to import database: {}", e),
        })?;
        Ok(())
    }
}

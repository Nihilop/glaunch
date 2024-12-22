use crate::utils::AppError;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use crate::utils::AppPaths;
use winreg::enums::*;
use winreg::RegKey;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::Foundation::HWND;

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
    paths: AppPaths,
}

impl SettingsManager {
    pub fn new(app_handle: &AppHandle) -> Result<Self, AppError> {
        let paths = AppPaths::new(app_handle)?;
        let settings_path = paths.get_settings_path();

        let settings = if settings_path.exists() {
            let content = fs::read_to_string(&settings_path).map_err(|e| AppError {
                message: format!("Failed to read settings file: {}", e),
            })?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            let default_settings = AppSettings::default();
            let content = serde_json::to_string_pretty(&default_settings).map_err(|e| AppError {
                message: format!("Failed to serialize default settings: {}", e),
            })?;
            fs::write(&settings_path, content).map_err(|e| AppError {
                message: format!("Failed to write default settings file: {}", e),
            })?;
            default_settings
        };

        Ok(Self {
            settings_path,
            settings,
            paths,
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
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = r"Software\Microsoft\Windows\CurrentVersion\Run";
        let (key, _) = hkcu.create_subkey(path).map_err(|e| AppError {
            message: format!("Failed to open/create registry key: {}", e),
        })?;

        let current_exe = env::current_exe().map_err(|e| AppError {
            message: format!("Failed to get current exe path: {}", e),
        })?;

        if self.settings.start_with_windows {
            key.set_value(
                "GLaunch",
                &current_exe.to_string_lossy().to_string(),
            ).map_err(|e| AppError {
                message: format!("Failed to set registry value: {}", e),
            })?;
        } else {
            // EnumValues est un itérateur directement - pas besoin de match
            let mut found = false;
            for value in key.enum_values() {
                if let Ok((name, _)) = value {
                    if name == "GLaunch" {
                        found = true;
                        break;
                    }
                }
            }

            if found {
                key.delete_value("GLaunch").map_err(|e| AppError {
                    message: format!("Failed to delete registry value: {}", e),
                })?;
            }
        }

        Ok(())
    }

    pub fn should_minimize_to_tray(&self, hwnd: HWND) -> bool {
        if !self.settings.minimize_to_tray {
            return false;
        }

        unsafe {
            let style = GetWindowLongW(hwnd, GWL_STYLE);
            if (style as u32 & WS_MINIMIZE.0) != 0 {
                return false;
            }
        }

        true
    }

    pub fn export_database(&self, export_path: PathBuf) -> Result<(), AppError> {
        // Utiliser le chemin de la base de données depuis AppPaths
        let db_path = self.paths.get_database_path();

        if !db_path.exists() {
            return Err(AppError {
                message: "Database file not found".to_string(),
            });
        }

        fs::copy(&db_path, export_path).map_err(|e| AppError {
            message: format!("Failed to export database: {}", e),
        })?;
        Ok(())
    }

    pub fn import_database(&self, import_path: PathBuf) -> Result<(), AppError> {
        // Utiliser le chemin de la base de données depuis AppPaths
        let db_path = self.paths.get_database_path();
        let db_dir = db_path.parent().ok_or_else(|| AppError {
            message: "Failed to get database directory".to_string(),
        })?;

        // Créer une sauvegarde
        if db_path.exists() {
            let backup_path = db_dir.join("games.db.backup");
            fs::copy(&db_path, &backup_path).map_err(|e| AppError {
                message: format!("Failed to create database backup: {}", e),
            })?;
        }

        fs::copy(import_path, &db_path).map_err(|e| AppError {
            message: format!("Failed to import database: {}", e),
        })?;
        Ok(())
    }
}

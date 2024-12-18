use crate::services::updater::UpdateStatus;
use crate::utils::settings::{AppSettings, SettingsManager};
use std::path::PathBuf;

#[tauri::command]
pub async fn get_app_settings() -> Result<AppSettings, String> {
    let settings_manager = SettingsManager::new().map_err(|e| e.message)?;
    Ok(settings_manager.get_settings().clone())
}

#[tauri::command]
pub async fn save_app_settings(settings: AppSettings) -> Result<(), String> {
    let mut settings_manager = SettingsManager::new().map_err(|e| e.message)?;
    settings_manager
        .update_settings(settings)
        .map_err(|e| e.message)
}

#[tauri::command]
pub async fn export_db(path: String) -> Result<(), String> {
    let settings_manager = SettingsManager::new().map_err(|e| e.message)?;
    settings_manager
        .export_database(PathBuf::from(path))
        .map_err(|e| e.message)
}

#[tauri::command]
pub async fn import_db(path: String) -> Result<(), String> {
    let settings_manager = SettingsManager::new().map_err(|e| e.message)?;
    settings_manager
        .import_database(PathBuf::from(path))
        .map_err(|e| e.message)
}

#[tauri::command]
pub async fn check_for_updates(app_handle: tauri::AppHandle) -> Result<UpdateStatus, String> {
    let service = crate::services::updater::UpdateService::new(env!("CARGO_PKG_VERSION"))
        .map_err(|e| e.message)?;
    service.check_for_updates().await.map_err(|e| e.message)
}

#[tauri::command]
pub async fn install_update(app_handle: tauri::AppHandle) -> Result<(), String> {
    let service = crate::services::updater::UpdateService::new(env!("CARGO_PKG_VERSION"))
        .map_err(|e| e.message)?;
    service.download_update().await.map_err(|e| e.message)
}

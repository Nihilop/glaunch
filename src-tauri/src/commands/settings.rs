use tauri_plugin_updater::UpdaterExt;
use tauri::AppHandle;
use crate::models::UpdateStatus;
use crate::utils::settings::{AppSettings, SettingsManager};
use tauri::Emitter;
use std::path::PathBuf;
use serde::Serialize;

#[derive(Serialize, Clone)]
struct DownloadProgress {
    downloaded_bytes: usize,
    total_bytes: Option<u64>,
    progress_percent: f64,
}

#[tauri::command]
pub async fn get_app_settings(app_handle: tauri::AppHandle) -> Result<AppSettings, String> {
    let settings_manager = SettingsManager::new(&app_handle).map_err(|e| e.message)?;
    Ok(settings_manager.get_settings().clone())
}

#[tauri::command]
pub async fn save_app_settings(app_handle: tauri::AppHandle, settings: AppSettings) -> Result<(), String> {
    let mut settings_manager = SettingsManager::new(&app_handle).map_err(|e| e.message)?;
    settings_manager
        .update_settings(settings)
        .map_err(|e| e.message)
}

#[tauri::command]
pub async fn export_db(app_handle: tauri::AppHandle, path: String) -> Result<(), String> {
    let settings_manager = SettingsManager::new(&app_handle).map_err(|e| e.message)?;
    settings_manager
        .export_database(PathBuf::from(path))
        .map_err(|e| e.message)
}

#[tauri::command]
pub async fn import_db(app_handle: tauri::AppHandle, path: String) -> Result<(), String> {
    let settings_manager = SettingsManager::new(&app_handle).map_err(|e| e.message)?;
    settings_manager
        .import_database(PathBuf::from(path))
        .map_err(|e| e.message)
}

#[tauri::command]
pub async fn check_for_updates(app_handle: AppHandle) -> Result<UpdateStatus, String> {
    let updater = app_handle.updater().map_err(|e| e.to_string())?;
    let update = updater.check().await.map_err(|e| e.to_string())?;

    if let Some(update) = update {
        Ok(UpdateStatus {
            current_version: app_handle.package_info().version.to_string(),
            latest_version: Some(update.version),
            update_available: true,
            last_check: Some(chrono::Utc::now().timestamp()),
            download_url: None,
            release_notes: update.body,
            checking: false,
        })
    } else {
        Ok(UpdateStatus {
            current_version: app_handle.package_info().version.to_string(),
            latest_version: None,
            update_available: false,
            last_check: Some(chrono::Utc::now().timestamp()),
            download_url: None,
            release_notes: None,
            checking: false,
        })
    }
}

#[tauri::command]
pub async fn install_update(app_handle: AppHandle) -> Result<(), String> {
    let updater = app_handle.updater().map_err(|e| e.to_string())?;

    match updater.check().await {
        Ok(Some(update)) => {
            // Clone du app_handle pour les callbacks
            let app_handle_clone = app_handle.clone();

            // Callback de progression du téléchargement
            let on_chunk = move |downloaded: usize, total: Option<u64>| {
                let progress = total.map_or(0.0, |total| {
                  downloaded as f64 / total as f64 * 100.0
                });

                let _ = app_handle_clone.emit(
                    "update-progress",
                    DownloadProgress {
                        downloaded_bytes: downloaded,
                        total_bytes: total,
                        progress_percent: progress,
                    },
                );
            };

            // Callback de fin de téléchargement
            let app_handle_clone = app_handle.clone();
            let on_download_finish = move || {
                let _ = app_handle_clone.emit("update-downloaded", ());
            };

            // Informer le frontend que le téléchargement commence
            let _ = app_handle.emit("update-starting", ());

            // Lancer le téléchargement et l'installation
            update.download_and_install(on_chunk, on_download_finish)
                .await
                .map_err(|e| e.to_string())
        }
        Ok(None) => Err("No update available".to_string()),
        Err(e) => Err(e.to_string())
    }
}
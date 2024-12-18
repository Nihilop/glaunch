use crate::OverlayState;
use tauri::{Emitter, Manager};

#[tauri::command]
pub async fn toggle_overlay(state: tauri::State<'_, OverlayState>) -> Result<(), String> {
    let overlay_lock = state.overlay.clone();

    if let Ok(mut overlay_guard) = overlay_lock.lock() {
        if let Some(overlay) = overlay_guard.as_mut() {
            if let Some(window) = overlay.app.get_webview_window("overlay") {
                if overlay.is_visible {
                    window.hide().map_err(|e| e.to_string())?;
                    overlay.is_visible = false;
                } else {
                    window.show().map_err(|e| e.to_string())?;
                    overlay.is_visible = true;
                }
            }
        }
    }
    Ok(())
}

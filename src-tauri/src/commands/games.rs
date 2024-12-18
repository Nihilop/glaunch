use crate::AppState;
use crate::services::IgdbSearchResult;
use crate::models::CustomGameConfig;
use crate::models::Game;
use std::path::PathBuf;

#[tauri::command]
pub async fn scan_games(
    state: tauri::State<'_, AppState>,
    use_cache: bool,
) -> Result<Vec<Game>, String> {
    state.game_manager
        .scan_all_platforms(use_cache)
        .await
        .map_err(|e| e.message)
}

#[tauri::command]
pub async fn update_game_metadata(
    state: tauri::State<'_, AppState>,
    game_id: String,
) -> Result<(), String> {
    state
        .game_manager
        .update_game_metadata(&game_id)
        .await
        .map_err(|e| e.message)
}

#[tauri::command]
pub async fn get_game(
    state: tauri::State<'_, AppState>,
    game_id: String,
) -> Result<Option<Game>, String> {
    state
        .game_manager
        .get_game(&game_id)
        .await
        .map_err(|e| e.message)
}

#[tauri::command]
pub async fn launch_game(game_id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state
        .game_manager
        .launch_game(&game_id)
        .await
        .map_err(|e| e.message)
}

#[tauri::command]
pub async fn get_active_game(state: tauri::State<'_, AppState>) -> Result<Option<String>, String> {
    Ok(state
        .game_monitor
        .get_active_game()
        .map(|session| session.game_id))
}

#[tauri::command]
pub async fn search_igdb_games(
    state: tauri::State<'_, AppState>,
    query: String,
) -> Result<Vec<IgdbSearchResult>, String> {
    state
        .game_manager
        .metadata_service
        .search_games(&query)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_game_with_igdb(
    state: tauri::State<'_, AppState>,
    game_id: String,
    igdb_id: i64,
) -> Result<(), String> {
    state
        .game_manager
        .metadata_service
        .update_with_igdb_id(&game_id, igdb_id)
        .await
        .map_err(|e| e.message)
}

#[tauri::command]
pub async fn add_custom_game(
    state: tauri::State<'_, AppState>,
    title: String,
    executable_path: String,
    install_path: String,
    platform: String,
    icon_path: Option<String>,
) -> Result<Game, String> {
    state
        .game_manager
        .add_custom_game(CustomGameConfig {
            title,
            executable_path,
            install_path,
            platform,
            icon_path,
        })
        .await
        .map_err(|e| e.message)
}

#[tauri::command]
pub async fn delete_game(
    state: tauri::State<'_, AppState>,
    game_id: String,
) -> Result<(), String> {
    state
        .game_manager
        .delete_game(&game_id)
        .await
        .map_err(|e| e.message)
}
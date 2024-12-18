use crate::api::{BattleNetApi, EpicApi, SteamApi};
use crate::auth::epic::EpicTokenResponse;
use crate::auth::EpicAuth;
use crate::utils::AppError;

/// Commandes API Steam
#[tauri::command]
pub async fn get_steam_profile(steam_id: String) -> Result<String, String> {
    let api_key = std::env::var("STEAM_API_KEY")
        .map_err(|_| "STEAM_API_KEY not set")?;

    let api = SteamApi::new(api_key);
    let profile = api.get_profile(&steam_id)
        .await
        .map_err(|e| e.message)?;

    serde_json::to_string(&profile)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_steam_friends(steam_id: String) -> Result<String, String> {
    let api_key = std::env::var("STEAM_API_KEY")
        .map_err(|_| "STEAM_API_KEY not set")?;

    let api = SteamApi::new(api_key);
    let friends = api.get_friends(&steam_id)
        .await
        .map_err(|e| e.message)?;

    serde_json::to_string(&friends)
        .map_err(|e| e.to_string())
}

/// Commandes API Epic
#[tauri::command]
pub async fn exchange_epic_code(code: String) -> Result<String, String> {
    let client_id = std::env::var("EPIC_CLIENT_ID")
        .map_err(|_| "EPIC_CLIENT_ID not set")?;
    let client_id_secret = std::env::var("EPIC_CLIENT_ID_SECRET")
        .map_err(|_| "EPIC_CLIENT_ID_SECRET not set")?;

    let auth = EpicAuth::new(client_id, client_id_secret);
    let token_response = auth.exchange_code(&code)
        .await
        .map_err(|e| e.message)?;

    serde_json::to_string(&token_response)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_epic_profile(token: String, accountId: String) -> Result<String, String> {
    let api = EpicApi::new(token);
    let profile = api.get_profile(accountId)
        .await
        .map_err(|e| e.message)?;

    serde_json::to_string(&profile)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_epic_friends(token: String) -> Result<String, String> {
    let api = EpicApi::new(token);
    let friends = api.get_friends()
        .await
        .map_err(|e| e.message)?;

    serde_json::to_string(&friends)
        .map_err(|e| e.to_string())
}

/// Commandes API Battle.net
#[tauri::command]
pub async fn get_battlenet_profile(token: String) -> Result<String, String> {
    let api = BattleNetApi::new(token);
    let profile = api.get_profile()
        .await
        .map_err(|e| e.message)?;

    serde_json::to_string(&profile)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_battlenet_friends(token: String) -> Result<String, String> {
    let api = BattleNetApi::new(token);
    let friends = api.get_friends()
        .await
        .map_err(|e| e.message)?;

    serde_json::to_string(&friends)
        .map_err(|e| e.to_string())
}
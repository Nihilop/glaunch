use crate::auth::epic::EpicTokenResponse;
use crate::auth::{BattleNetAuth, EpicAuth, SteamAuth};
use crate::utils::AppError;

/// Commandes d'authentification Steam
#[tauri::command]
pub async fn auth_steam() -> Result<String, String> {
    let api_key = std::env::var("STEAM_API_KEY").map_err(|_| "STEAM_API_KEY not set")?;

    let auth = SteamAuth::new(api_key);
    Ok(auth.get_auth_url())
}

#[tauri::command]
pub async fn auth_steam_callback(identity: String) -> Result<String, String> {
    let api_key = std::env::var("STEAM_API_KEY").map_err(|_| "STEAM_API_KEY not set")?;

    let auth = SteamAuth::new(api_key);
    let steam_id = auth
        .extract_steam_id(&identity)
        .ok_or("Invalid Steam identity")?;

    Ok(steam_id)
}

/// Commandes d'authentification Epic
#[tauri::command]
pub async fn auth_epic() -> Result<String, String> {
    let client_id = std::env::var("EPIC_CLIENT_ID").map_err(|_| "EPIC_CLIENT_ID not set")?;
    let client_id_secret =
        std::env::var("EPIC_CLIENT_ID_SECRET").map_err(|_| "EPIC_CLIENT_ID_SECRET not set")?;

    let auth = EpicAuth::new(client_id, client_id_secret);
    Ok(auth.get_auth_url())
}

#[tauri::command]
pub async fn auth_epic_callback(code: String) -> Result<String, String> {
    let client_id = std::env::var("EPIC_CLIENT_ID").map_err(|_| "EPIC_CLIENT_ID not set")?;
    let client_id_secret =
        std::env::var("EPIC_CLIENT_ID_SECRET").map_err(|_| "EPIC_CLIENT_ID_SECRET not set")?;

    let auth = EpicAuth::new(client_id, client_id_secret);
    let token_response = auth.exchange_code(&code).await.map_err(|e| e.message)?;

    serde_json::to_string(&token_response).map_err(|e| e.to_string())
}

/// Commandes d'authentification Battle.net
#[tauri::command]
pub async fn auth_battlenet_callback(code: String) -> Result<String, String> {
    let client_id =
        std::env::var("BATTLENET_CLIENT_ID").map_err(|_| "BATTLENET_CLIENT_ID not set")?;
    let client_secret =
        std::env::var("BATTLENET_CLIENT_SECRET").map_err(|_| "BATTLENET_CLIENT_SECRET not set")?;

    let auth = BattleNetAuth::new(client_id, client_secret);

    // Exchange code for token response
    let token_response = auth.exchange_code(&code).await.map_err(|e| e.message)?;

    // Serialize the token response to string
    serde_json::to_string(&token_response).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn auth_battlenet() -> Result<String, String> {
    let client_id =
        std::env::var("BATTLENET_CLIENT_ID").map_err(|_| "BATTLENET_CLIENT_ID not set")?;
    let client_secret =
        std::env::var("BATTLENET_CLIENT_SECRET").map_err(|_| "BATTLENET_CLIENT_SECRET not set")?;

    let auth = BattleNetAuth::new(client_id, client_secret);
    Ok(auth.get_auth_url())
}

use crate::utils::AppError;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct BattleNetProfile {
    #[serde(rename = "id")]
    pub id: i64,
    pub battletag: String,
    pub sub: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BattleNetFriend {
    pub id: String,
    pub battletag: String,
    pub status: String,
    pub game_id: Option<String>,
    pub game_name: Option<String>,
}

pub struct BattleNetApi {
    client: Client,
    access_token: String,
}

impl BattleNetApi {
    pub fn new(access_token: String) -> Self {
        Self {
            client: Client::new(),
            access_token,
        }
    }

    pub async fn get_profile(&self) -> Result<BattleNetProfile, AppError> {
        // On utilise le token stocké dans l'instance
        let response = self.client
            .get("https://oauth.battle.net/oauth/userinfo")
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch profile: {}", e)
            })?;

        let text = response.text().await?;

        serde_json::from_str(&text)
            .map_err(|e| AppError {
                message: format!("Failed to parse profile: {}", e)
            })
    }

    pub async fn get_friends(&self) -> Result<Vec<BattleNetFriend>, AppError> {
        let response = self.client
            .get("https://us.api.blizzard.com/social/friends")
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch friends: {}", e)
            })?;

        let response_data: serde_json::Value = response.json().await
            .map_err(|e| AppError {
                message: format!("Failed to parse friends response: {}", e)
            })?;

        // Extraire le tableau d'amis de la réponse
        let friends = response_data["friends"]
            .as_array()
            .ok_or_else(|| AppError {
                message: "Invalid friends response format".to_string()
            })?;

        // Désérialiser le tableau d'amis
        serde_json::from_value(serde_json::Value::Array(friends.to_vec()))
            .map_err(|e| AppError {
                message: format!("Failed to parse friends data: {}", e)
            })
    }
}
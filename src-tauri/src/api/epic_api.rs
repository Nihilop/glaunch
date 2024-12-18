use crate::utils::AppError;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EpicProfile {
    pub accountId: String,
    pub displayName: String,
    pub preferredLanguage: String,
    pub linkedAccounts: Vec<LinkedAccount>,
    pub country: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinkedAccount {
    pub identityProviderId: String,
    pub displayName: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EpicFriend {
    pub accountId: String,
    pub displayName: String,
    pub status: String,
    pub presence: Option<EpicPresence>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EpicPresence {
    pub status: String,
    pub game: Option<String>,
}

pub struct EpicApi {
    client: Client,
    access_token: String,
}

impl EpicApi {
    pub fn new(access_token: String) -> Self {
        Self {
            client: Client::new(),
            access_token,
        }
    }

    pub async fn get_profile(&self, accountId: String) -> Result<EpicProfile, AppError> {
        let url = format!(
            "https://api.epicgames.dev/epic/id/v2/accounts?accountId={}",
            accountId
        );
        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch Epic profile: {}", e),
            })?;

        let text = response.text().await?;

        // Désérialiser le tableau JSON
        let profiles: Vec<EpicProfile> = serde_json::from_str(&text).map_err(|e| AppError {
            message: format!("Failed to parse profile: {}", e),
        })?;

        // Retourner le premier élément du tableau
        if let Some(profile) = profiles.into_iter().next() {
            Ok(profile)
        } else {
            Err(AppError {
                message: "No profile found in the response".to_string(),
            })
        }
    }

    pub async fn get_friends(&self) -> Result<Vec<EpicFriend>, AppError> {
        let response = self
            .client
            .get("https://api.epicgames.com/friends/public/friends")
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch Epic friends: {}", e),
            })?;

        response.json().await.map_err(|e| AppError {
            message: format!("Failed to parse Epic friends: {}", e),
        })
    }
}

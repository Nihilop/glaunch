use crate::utils::AppError;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u32,
    pub sub: String,
    pub scope: String,
}

pub struct BattleNetAuth {
    client: Client,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
}

fn deserialize_number_to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber {
        String(String),
        Number(i64),
    }

    Ok(match StringOrNumber::deserialize(deserializer)? {
        StringOrNumber::String(s) => s,
        StringOrNumber::Number(n) => n.to_string(),
    })
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BattleNetProfile {
    #[serde(deserialize_with = "deserialize_number_to_string")]
    pub id: String,
    pub battletag: String,
    #[serde(rename = "sub")]
    pub account_id: String,
}

impl BattleNetAuth {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client: Client::new(),
            client_id,
            client_secret,
            redirect_uri: "http://localhost:11111/auth/battlenet/callback".to_string(),
        }
    }

    pub fn get_auth_url(&self) -> String {
        format!(
            "https://oauth.battle.net/authorize?\
            client_id={}&\
            response_type=code&\
            redirect_uri={}&\
            state=test&\
            scope=openid+profile",
            self.client_id, self.redirect_uri
        )
    }

    pub async fn exchange_code(&self, code: &str) -> Result<TokenResponse, AppError> {
        let response = self
            .client
            .post("https://oauth.battle.net/token")
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", &self.redirect_uri),
            ])
            .send()
            .await
            .map_err(|e| AppError {
                message: format!("Failed to exchange code: {}", e),
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError {
                message: format!("Battle.net token request failed: {}", error_text),
            });
        }

        response.json().await.map_err(|e| AppError {
            message: format!("Failed to parse token response: {}", e),
        })
    }

    pub async fn get_profile(&self, token: &str) -> Result<BattleNetProfile, AppError> {
        let response = self
            .client
            .get("https://oauth.battle.net/oauth/userinfo")
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch profile: {}", e),
            })?;

        // Log de la réponse brute pour debug
        let text = response.text().await.map_err(|e| AppError {
            message: format!("Failed to get response text: {}", e),
        })?;

        serde_json::from_str(&text).map_err(|e| AppError {
            message: format!("Failed to parse profile: {}", e),
        })
    }
}

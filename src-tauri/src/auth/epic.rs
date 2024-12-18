use crate::utils::AppError;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;

#[derive(Deserialize, Serialize, Debug)]
pub struct EpicTokenResponse {
    pub scope: String,
    pub token_type: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub expires_at: String,
    pub refresh_expires_in: u64,
    pub refresh_expires_at: String,
    pub account_id: String,
    pub client_id: String,
    pub application_id: String,
    pub acr: String,
    pub auth_time: String,
}

pub struct EpicAuth {
    client: Client,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
}

impl EpicAuth {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client: Client::new(),
            client_id,
            client_secret,
            redirect_uri: "http://localhost:11111/auth/epic/callback".to_string(),
        }
    }

    pub fn get_auth_url(&self) -> String {
        let mut url = Url::parse("https://www.epicgames.com/id/authorize").unwrap();
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("response_type", "code")
            .append_pair("redirect_uri", &self.redirect_uri);

        url.to_string()
    }

    pub async fn exchange_code(&self, code: &str) -> Result<EpicTokenResponse, AppError> {
        let auth_header = format!(
            "Basic {}",
            STANDARD.encode(format!("{}:{}", self.client_id, self.client_secret))
        );

        let response = self.client
            .post("https://api.epicgames.dev/epic/oauth/v2/token")
            .header("Authorization", auth_header)
            .form(&[
                ("grant_type", "authorization_code"),
                ("code", code),
                ("redirect_uri", &self.redirect_uri),
            ])
            .send()
            .await
            .map_err(|e| AppError {
                message: format!("Failed to exchange Epic code: {}", e)
            })?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError {
                message: format!("Epic token request failed: {}", error_text)
            });
        }

        let response_text = response.text().await.map_err(|e| AppError {
            message: format!("Failed to read response text: {}", e)
        })?;

        let token_data: EpicTokenResponse = serde_json::from_str(&response_text)
            .map_err(|e| AppError {
                message: format!("Failed to parse Epic token response: {}", e)
            })?;

        Ok(token_data)
    }
}
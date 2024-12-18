use crate::utils::AppError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct SteamProfile {
    pub steamid: String,
    pub personaname: String,
    pub profileurl: String,
    pub avatar: String,
    pub avatarfull: String,
    pub personastate: i32,
    pub gameextrainfo: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SteamFriend {
    pub steamid: String,
    pub relationship: String,
    pub friend_since: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameMediaInfo {
    pub thumbnail: Option<String>,
    pub cover: Option<String>,
    pub screenshots: Vec<String>,
    pub description: Option<String>,
}

pub struct SteamApi {
    client: Client,
    api_key: String,
    store_api_url: String,
}

impl SteamApi {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            store_api_url: "https://store.steampowered.com/api".to_string(),
        }
    }

    pub async fn get_profile(&self, steam_id: &str) -> Result<SteamProfile, AppError> {
        let url = format!(
            "https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v2/?key={}&steamids={}",
            self.api_key, steam_id
        );

        let response = self.client.get(&url)
            .send()
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch Steam profile: {}", e)
            })?;

        let data: serde_json::Value = response.json().await
            .map_err(|e| AppError {
                message: format!("Failed to parse Steam profile: {}", e)
            })?;

        serde_json::from_value(data["response"]["players"][0].clone())
            .map_err(|e| AppError {
                message: format!("Failed to parse player data: {}", e)
            })
    }

    pub async fn get_friends(&self, steam_id: &str) -> Result<Vec<SteamFriend>, AppError> {
        let url = format!(
            "https://api.steampowered.com/ISteamUser/GetFriendList/v1/?key={}&steamid={}&relationship=friend",
            self.api_key, steam_id
        );

        let response = self.client.get(&url)
            .send()
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch Steam friends: {}", e)
            })?;

        let data: serde_json::Value = response.json().await
            .map_err(|e| AppError {
                message: format!("Failed to parse Steam friends: {}", e)
            })?;

        serde_json::from_value(data["friendslist"]["friends"].clone())
            .map_err(|e| AppError {
                message: format!("Failed to parse friends data: {}", e)
            })
    }

    pub async fn get_app_details(&self, app_id: &str) -> Result<GameMediaInfo, AppError> {
        let url = format!("{}/appdetails?appids={}", self.store_api_url, app_id);

        #[derive(Deserialize)]
        struct SteamAppDetails {
            data: Option<SteamAppData>,
        }

        #[derive(Deserialize)]
        struct SteamAppData {
            name: String,
            header_image: String,
            background: Option<String>,
            description: Option<String>,
            screenshots: Option<Vec<Screenshot>>,
        }

        #[derive(Deserialize)]
        struct Screenshot {
            path_full: String,
        }

        let response = self.client.get(&url)
            .send()
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch Steam app details: {}", e)
            })?;

        let mut details: HashMap<String, SteamAppDetails> = response.json()
            .await
            .map_err(|e| AppError {
                message: format!("Failed to parse Steam response: {}", e)
            })?;

        let app_details = details
            .remove(app_id)
            .and_then(|d| d.data)
            .ok_or_else(|| AppError {
                message: format!("No data found for app_id: {}", app_id)
            })?;

        Ok(GameMediaInfo {
            thumbnail: Some(app_details.header_image),
            cover: app_details.background,
            screenshots: app_details.screenshots
                .map(|s| s.iter().map(|ss| ss.path_full.clone()).collect())
                .unwrap_or_default(),
            description: app_details.description,
        })
    }
}
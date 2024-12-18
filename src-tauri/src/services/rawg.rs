use crate::models::{GameMedia, GameMetadata};
use crate::utils::cache::MediaCache;
use crate::utils::AppError;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

const API_BASE_URL: &str = "https://api.rawg.io/api";

#[derive(Debug, Deserialize)]
pub struct RawgGame {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub description_raw: Option<String>,
    pub released: Option<String>,
    pub background_image: Option<String>,
    pub background_image_additional: Option<String>,
    pub website: Option<String>,
    pub metacritic: Option<i32>,
    pub developers: Vec<RawgCompany>,
    pub publishers: Vec<RawgCompany>,
    pub genres: Vec<RawgGenre>,
    pub tags: Vec<RawgTag>,
    pub screenshots: Option<RawgScreenshots>,
}

#[derive(Debug, Deserialize)]
pub struct RawgCompany {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct RawgGenre {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct RawgTag {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct RawgScreenshots {
    pub results: Vec<RawgScreenshot>,
}

#[derive(Debug, Deserialize)]
pub struct RawgScreenshot {
    pub id: i32,
    pub image: String,
}

pub struct RawgService {
    pub client: Client,
    pub api_key: String,
}

impl RawgService {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn search_game(&self, name: &str) -> Result<RawgGame, AppError> {
        let url = format!(
            "{}/games?search={}&key={}",
            API_BASE_URL,
            urlencoding::encode(name),
            self.api_key
        );

        // Faire la requête
        let response = self.client.get(&url).send().await.map_err(|e| AppError {
            message: format!("Failed to fetch from RAWG: {}", e),
        })?;

        // Récupérer le body de la réponse en tant que texte d'abord
        let response_text = response.text().await.map_err(|e| AppError {
            message: format!("Failed to read RAWG response: {}", e),
        })?;

        // Parser la réponse JSON
        let games: serde_json::Value =
            serde_json::from_str(&response_text).map_err(|e| AppError {
                message: format!("Failed to parse RAWG response: {}", e),
            })?;

        // Log du nombre de résultats
        let results = games["results"].as_array();

        // Prendre le premier résultat
        let game_id = games["results"][0]["id"].as_i64().ok_or_else(|| AppError {
            message: "No game found".to_string(),
        })?;

        // Obtenir les détails
        self.get_game_details(game_id as i32).await
    }

    async fn get_game_details(&self, game_id: i32) -> Result<RawgGame, AppError> {
        let url = format!("{}/games/{}?key={}", API_BASE_URL, game_id, self.api_key);

        let response = self.client.get(&url).send().await.map_err(|e| AppError {
            message: format!("Failed to fetch game details: {}", e),
        })?;

        let response_text = response.text().await.map_err(|e| AppError {
            message: format!("Failed to read game details: {}", e),
        })?;

        serde_json::from_str(&response_text).map_err(|e| AppError {
            message: format!("Failed to parse game details: {}", e),
        })
    }
    pub async fn cache_game_media(
        &self,
        game_id: &str,
        rawg_game: &RawgGame,
    ) -> Result<GameMedia, AppError> {
        let media_cache = MediaCache::new()?;
        let cache_duration = Duration::from_secs(7 * 24 * 60 * 60);

        let background = if let Some(url) = &rawg_game.background_image {
            let path = media_cache.get_or_download(url, cache_duration).await?;
            Some(path)
        } else {
            None
        };

        let thumbnail = if let Some(url) = &rawg_game.background_image_additional {
            let path = media_cache.get_or_download(url, cache_duration).await?;
            Some(path)
        } else {
            background.clone()
        };

        let mut screenshots = Vec::new();
        if let Some(screenshots_data) = &rawg_game.screenshots {
            for screenshot in &screenshots_data.results {
                if let Ok(path) = media_cache
                    .get_or_download(&screenshot.image, cache_duration)
                    .await
                {
                    screenshots.push(path);
                }
            }
        }

        Ok(GameMedia {
            thumbnail,
            cover: background.clone(),
            background,
            screenshots,
            icon: None,
            logo: None,
        })
    }

    pub async fn convert_to_metadata(
        &self,
        game_id: &str,
        rawg_game: RawgGame,
    ) -> Result<(GameMetadata, GameMedia), AppError> {
        let media = self.cache_game_media(game_id, &rawg_game).await?;

        let metadata = GameMetadata {
            title: rawg_game.name,
            description: rawg_game.description_raw,
            developer: rawg_game.developers.first().map(|d| d.name.clone()),
            publisher: rawg_game.publishers.first().map(|p| p.name.clone()),
            release_date: rawg_game.released,
            genres: rawg_game.genres.into_iter().map(|g| g.name).collect(),
            tags: rawg_game.tags.into_iter().map(|t| t.name).collect(),
            media: Some(media.clone()),
        };

        Ok((metadata, media))
    }
}

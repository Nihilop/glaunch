use crate::models::{GameMedia, GameMetadata};
use crate::utils::{cache::MediaCache, AppError};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tauri::AppHandle;
use crate::log_info;

const IGDB_API_URL: &str = "https://api.igdb.com/v4";
const RATE_LIMIT_DELAY: Duration = Duration::from_millis(250); // 4 requêtes par seconde max
static mut LAST_REQUEST: Option<Instant> = None;

#[derive(Debug, Deserialize)]
pub struct IgdbGame {
    pub id: i64,
    pub name: String,
    pub summary: Option<String>,
    pub cover: Option<IgdbImage>,
    pub artworks: Option<Vec<IgdbImage>>,
    pub screenshots: Option<Vec<IgdbImage>>,
    pub first_release_date: Option<i64>,
    pub involved_companies: Option<Vec<IgdbCompany>>,
    pub genres: Option<Vec<IgdbGenre>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IgdbCompany {
    pub company: IgdbCompanyInfo,
    pub developer: bool,
    pub publisher: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IgdbCompanyInfo {
    pub id: i64,
    pub name: String,
    pub logo: Option<IgdbImage>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IgdbImage {
    pub id: i64,
    pub image_id: String,
    #[serde(rename = "url")]
    pub url_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IgdbGenre {
    pub id: i64,
    pub name: String,
}

pub struct IgdbService {
    client: Client,
    client_id: String,
    access_token: String,
    media_cache: MediaCache,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IgdbSearchResult {
    pub id: i64,
    pub name: String,
    pub release_date: Option<String>,
    pub cover_url: Option<String>,
    pub company: Option<String>,
}

impl IgdbService {
    pub fn new(client_id: String, access_token: String, app: &AppHandle) -> Result<Self, AppError> {
        if client_id.is_empty() || access_token.is_empty() {
            log_info!("Creating limited IGDB service without authentication");
            return Ok(Self {
                client: Client::new(),
                client_id,
                access_token,
                media_cache: MediaCache::new(app)?,
            });
        }
        Ok(Self {
            client: Client::new(),
            client_id,
            access_token,
            media_cache: MediaCache::new(app)?,
        })
    }

    pub async fn search_games(&self, query: &str) -> Result<Vec<IgdbSearchResult>, AppError> {
        // Vérifier si le service est authentifié
        if self.client_id.is_empty() || self.access_token.is_empty() {
            log_info!("IGDB search skipped: no authentication");
            return Ok(Vec::new());
        }
        Self::enforce_rate_limit().await;

        let body = format!(
            r#"search "{query}";
            fields name, summary, cover.*,
            artworks.*, screenshots.*,
            first_release_date,
            involved_companies.company.name,
            involved_companies.company.logo.*,
            involved_companies.developer,
            involved_companies.publisher,
            genres.*;
            limit 10;"#
        );

        let response = self
            .client
            .post(&format!("{}/games", IGDB_API_URL))
            .headers(self.get_headers())
            .body(body)
            .send()
            .await?;

        let games: Vec<IgdbGame> = response.json().await?;

        Ok(games
            .into_iter()
            .map(|game| IgdbSearchResult {
                id: game.id,
                name: game.name,
                release_date: game.first_release_date.map(|ts| {
                    chrono::DateTime::from_timestamp(ts, 0)
                        .map(|dt| dt.format("%Y-%m-%d").to_string())
                        .unwrap_or_default()
                }),
                cover_url: game.cover.and_then(|c| c.url_path.clone()),
                company: game
                    .involved_companies
                    .and_then(|companies| companies.first().cloned())
                    .map(|c| c.company.name.clone()),
            })
            .collect())
    }

    pub async fn get_game_by_id(&self, igdb_id: i64) -> Result<IgdbGame, AppError> {
        Self::enforce_rate_limit().await;

        let body = format!(
            r#"where id = {igdb_id};
            fields name, summary, cover.*,
            artworks.*, screenshots.*,
            first_release_date,
            involved_companies.company.name,
            involved_companies.company.logo.*,
            involved_companies.developer,
            involved_companies.publisher,
            genres.*;"#
        );

        let response = self
            .client
            .post(&format!("{}/games", IGDB_API_URL))
            .headers(self.get_headers())
            .body(body)
            .send()
            .await?;

        let mut games: Vec<IgdbGame> = response.json().await?;
        games.pop().ok_or_else(|| AppError {
            message: "Game not found".to_string(),
        })
    }

    async fn enforce_rate_limit() {
        unsafe {
            if let Some(last) = LAST_REQUEST {
                let elapsed = last.elapsed();
                if elapsed < RATE_LIMIT_DELAY {
                    sleep(RATE_LIMIT_DELAY - elapsed).await;
                }
            }
            LAST_REQUEST = Some(Instant::now());
        }
    }

    fn get_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("Client-ID", HeaderValue::from_str(&self.client_id).unwrap());
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.access_token)).unwrap(),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers
    }

    pub async fn search_game(&self, name: &str) -> Result<IgdbGame, AppError> {
        Self::enforce_rate_limit().await;

        let body = format!(
            r#"search "{name}";
            fields name, summary, cover.*,
            artworks.*, screenshots.*,
            first_release_date,
            involved_companies.company.name,
            involved_companies.company.logo.*,
            involved_companies.developer,
            involved_companies.publisher,
            genres.*;
            limit 1;"#
        );

        let response = self
            .client
            .post(&format!("{}/games", IGDB_API_URL))
            .headers(self.get_headers())
            .body(body)
            .send()
            .await
            .map_err(|e| AppError {
                message: format!("IGDB API request failed: {}", e),
            })?;

        let games: Vec<IgdbGame> = response.json().await.map_err(|e| AppError {
            message: format!("Failed to parse IGDB response: {}", e),
        })?;

        games.into_iter().next().ok_or_else(|| AppError {
            message: "No game found".to_string(),
        })
    }

    pub async fn get_media(&self, game: &IgdbGame) -> Result<GameMedia, AppError> {
        let cache_duration = Duration::from_secs(7 * 24 * 60 * 60);

        // Fonction helper pour construire les URLs d'images
        let build_image_url = |image_id: &str, size: &str| {
            format!(
                "https://images.igdb.com/igdb/image/upload/t_{}/{}.jpg",
                size, image_id
            )
        };

        // Récupérer le thumbnail depuis la cover
        let thumbnail = if let Some(cover) = &game.cover {
            let url = build_image_url(&cover.image_id, "cover_big");
            Some(
                self.media_cache
                    .get_or_download(&url, cache_duration)
                    .await?,
            )
        } else {
            None
        };

        // Récupérer le logo depuis les artworks
        let logo = if let Some(artworks) = &game.artworks {
            if let Some(artwork) = artworks.first() {
                let url = build_image_url(&artwork.image_id, "logo_med");
                Some(
                    self.media_cache
                        .get_or_download(&url, cache_duration)
                        .await?,
                )
            } else {
                None
            }
        } else {
            None
        };

        // Récupérer l'icône depuis le logo de la company
        let icon = if let Some(companies) = &game.involved_companies {
            if let Some(company) = companies.first() {
                if let Some(logo) = &company.company.logo {
                    let url = build_image_url(&logo.image_id, "thumb");
                    Some(
                        self.media_cache
                            .get_or_download(&url, cache_duration)
                            .await?,
                    )
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Récupérer les screenshots
        let mut screenshots = Vec::new();
        if let Some(screens) = &game.screenshots {
            for screen in screens {
                let url = build_image_url(&screen.image_id, "screenshot_big");
                if let Ok(path) = self.media_cache.get_or_download(&url, cache_duration).await {
                    screenshots.push(path);
                }
            }
        }

        let thumbnail_clone = thumbnail.clone();
        Ok(GameMedia {
            thumbnail,
            cover: thumbnail_clone,
            background: screenshots.first().cloned(),
            screenshots,
            icon,
            logo,
        })
    }
}

// Dans services/metadata.rs
use super::igdb::IgdbService;
use crate::db::Database;
use crate::models::{Game, GameMedia, GameMetadata};
use crate::services::igdb::IgdbGame;
use crate::services::IgdbSearchResult;
use crate::utils::AppError;
use std::sync::Arc;
use tauri::AppHandle;
use crate::log_info;
use crate::log_warn;
use crate::log_error;
use crate::log_debug;

pub struct MetadataService {
    database: Arc<Database>,
    igdb: Arc<IgdbService>,
}

impl MetadataService {
    pub fn new(database: Arc<Database>, app: AppHandle, client_id: String, client_secret: String) -> Result<Self, AppError> {
        // Obtenir le token d'accès IGDB - mais ne pas bloquer si ça échoue
        let access_token = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(Self::get_twitch_access_token(&client_id, &client_secret))
            .unwrap_or_else(|e| {
                log_warn!("Failed to get Twitch access token: {}. IGDB features will be limited.", e);
                String::new()
            });

        Ok(Self {
            database,
            igdb: Arc::new(IgdbService::new(client_id, access_token, &app)?),
        })
    }

    pub async fn update_with_igdb_id(&self, game_id: &str, igdb_id: i64) -> Result<(), AppError> {
        // Récupérer d'abord le jeu de la base de données
        let game = self
            .database
            .games()
            .get_game(game_id)
            .await?
            .ok_or_else(|| AppError {
                message: "Game not found in database".to_string(),
            })?;

        // Récupérer les données IGDB
        let igdb_game = self.igdb.get_game_by_id(igdb_id).await?;

        // Mettre à jour les métadonnées et médias
        let (metadata, media) = self.convert_to_metadata(&game.id, igdb_game).await?;

        // Sauvegarder en base
        self.database
            .metadata()
            .update_metadata(&game.id, &metadata, Some(&media))
            .await?;

        Ok(())
    }

    pub async fn search_games(&self, query: &str) -> Result<Vec<IgdbSearchResult>, AppError> {
        self.igdb.search_games(query).await
    }

    pub async fn convert_to_metadata(
        &self,
        game_id: &str,
        igdb_game: IgdbGame,
    ) -> Result<(GameMetadata, GameMedia), AppError> {
        let metadata = GameMetadata {
            title: igdb_game.name.clone(),
            description: igdb_game.summary.clone(),
            developer: igdb_game.involved_companies.as_ref().and_then(|companies| {
                companies
                    .iter()
                    .find(|c| c.developer)
                    .map(|c| c.company.name.clone())
            }),
            publisher: igdb_game.involved_companies.as_ref().and_then(|companies| {
                companies
                    .iter()
                    .find(|c| c.publisher)
                    .map(|c| c.company.name.clone())
            }),
            release_date: igdb_game.first_release_date.map(|ts| {
                chrono::DateTime::from_timestamp(ts, 0)
                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                    .unwrap_or_default()
            }),
            genres: igdb_game
                .genres
                .clone()
                .unwrap_or_default()
                .into_iter()
                .map(|genre| genre.name)
                .collect(),
            tags: Vec::new(),
            media: None,
        };

        let media = self.igdb.get_media(&igdb_game).await?;

        Ok((metadata, media))
    }

    async fn get_twitch_access_token(client_id: &str, client_secret: &str) -> Result<String, AppError> {
        log_debug!("Attempting to get Twitch access token...");

        let client = reqwest::Client::new();
        let response = client
            .post("https://id.twitch.tv/oauth2/token")
            .form(&[
                ("client_id", client_id),
                ("client_secret", client_secret),
                ("grant_type", "client_credentials"),
            ])
            .send()
            .await
            .map_err(|e| AppError {
                message: format!("Failed to get Twitch access token: {}", e),
            })?;

        // Log de la réponse HTTP
        log_debug!("Twitch response status: {}", response.status());

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            log_error!("Twitch auth failed. Response: {}", error_text);
            return Ok(String::new());
        }

        // Log du body de la réponse pour debug
        let response_text = response.text().await.map_err(|e| AppError {
            message: format!("Failed to read response body: {}", e),
        })?;
        log_debug!("Twitch response body: {}", response_text);

        let token_data: serde_json::Value = serde_json::from_str(&response_text).map_err(|e| AppError {
            message: format!("Failed to parse token response: {}", e),
        })?;

        if let Some(token) = token_data["access_token"].as_str() {
            log_debug!("Successfully retrieved Twitch access token");
            Ok(token.to_string())
        } else {
            log_error!("No access_token in response: {:?}", token_data);
            Ok(String::new())
        }
    }

    pub async fn update_metadata(&self, game: &mut Game) -> Result<(), AppError> {
        match self.igdb.search_game(&game.title).await {
            Ok(igdb_game) => {
                // Créer les métadonnées à partir des données IGDB
                let metadata = GameMetadata {
                    title: igdb_game.name.clone(),
                    description: igdb_game.summary.clone(),
                    developer: igdb_game.involved_companies.as_ref().and_then(|companies| {
                        companies
                            .iter()
                            .find(|c| c.developer)
                            .map(|c| c.company.name.clone())
                    }),
                    publisher: igdb_game.involved_companies.as_ref().and_then(|companies| {
                        companies
                            .iter()
                            .find(|c| c.publisher)
                            .map(|c| c.company.name.clone())
                    }),
                    release_date: igdb_game.first_release_date.map(|ts| {
                        chrono::DateTime::from_timestamp(ts, 0)
                            .map(|dt| dt.format("%Y-%m-%d").to_string())
                            .unwrap_or_default()
                    }),
                    genres: igdb_game
                        .genres
                        .as_ref()
                        .map(|genres| genres.iter().map(|g| g.name.clone()).collect())
                        .unwrap_or_default(),
                    tags: Vec::new(),
                    media: None,
                };

                // Récupérer les médias
                let media = self.igdb.get_media(&igdb_game).await?;

                self.database
                    .metadata()
                    .update_metadata(&game.id, &metadata, Some(&media))
                    .await?;

                // Mettre à jour le jeu en mémoire
                game.metadata = metadata;
                game.media = media;
                Ok(())
            }
            Err(e) => Ok(()),
        }
    }
}

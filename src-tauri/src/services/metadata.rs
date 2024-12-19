// Dans services/metadata.rs
use super::igdb::IgdbService;
use crate::db::Database;
use crate::models::{Game, GameMedia, GameMetadata};
use crate::services::igdb::IgdbGame;
use crate::services::IgdbSearchResult;
use crate::utils::AppError;
use std::sync::Arc;
use crate::log_info;

pub struct MetadataService {
    database: Arc<Database>,
    igdb: Arc<IgdbService>,
}

impl MetadataService {
    pub fn new(database: Arc<Database>, client_id: String, client_secret: String) -> Result<Self, AppError> {
        // Vérifier si les clés sont présentes
        if client_id.is_empty() || client_secret.is_empty() {
            log_info!("IGDB credentials not provided, metadata service will be limited");
            return Ok(Self {
                database,
                igdb: Arc::new(IgdbService::new(client_id, String::new())?),
            });
        }

        // Si les clés sont présentes, obtenir le token d'accès IGDB
        log_info!("Initializing IGDB service with credentials");
        let access_token = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(Self::get_twitch_access_token(&client_id, &client_secret))?;

        Ok(Self {
            database,
            igdb: Arc::new(IgdbService::new(client_id, access_token)?),
        })
    }

    pub async fn update_with_igdb_id(&self, game_id: &str, igdb_id: i64) -> Result<(), AppError> {
        // Récupérer d'abord le jeu de la base de données
        let mut game = self
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

    async fn get_twitch_access_token(
        client_id: &str,
        client_secret: &str,
    ) -> Result<String, AppError> {
        if client_id.is_empty() || client_secret.is_empty() {
            return Ok(String::new());
        }
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

        let token_data: serde_json::Value = response.json().await.map_err(|e| AppError {
            message: format!("Failed to parse token response: {}", e),
        })?;

        token_data["access_token"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| AppError {
                message: "No access token in response".to_string(),
            })
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

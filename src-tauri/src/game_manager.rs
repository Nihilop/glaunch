use crate::models::{Game, GameResult};
use crate::monitor::GameMonitor;
use crate::platforms::traits::GameScanner;
use crate::platforms::traits::MetadataProvider;
use crate::platforms::{battlenet, epic, steam, GamePlatform};
use tokio::process::Command;
use crate::services::MetadataService;
use crate::utils::AppError;
use crate::Database;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use tauri::AppHandle;
use tokio::sync::Mutex;
use std::path::Path;
use crate::models::GameStats;
use crate::models::GameMedia;
use crate::models::GameMetadata;
use crate::models::GameInstallation;
use crate::models::Platform;
use crate::models::CustomGameConfig;

pub struct GameCache {
    games: Vec<Game>,
    last_update: SystemTime,
}

pub struct GameManager {
    platforms: Vec<Arc<dyn GamePlatform>>,
    steam_platform: Arc<Mutex<steam::SteamPlatform>>,
    database: Arc<Database>,
    app_handle: AppHandle,
    game_monitor: Arc<GameMonitor>,
    pub metadata_service: Arc<MetadataService>,
}

impl GameManager {
    pub fn new(
        database: Arc<Database>,
        app_handle: AppHandle,
        game_monitor: Arc<GameMonitor>,
        igdb_client_id: String,
        igdb_client_secret: String
    ) -> Result<Self, AppError> {
        let steam_platform = Arc::new(Mutex::new(steam::SteamPlatform::new(database.clone())));
        let metadata_service = MetadataService::new(
            database.clone(),
            igdb_client_id,
            igdb_client_secret,
        )?;

        Ok(Self {
            platforms: vec![
                Arc::new(steam::SteamPlatform::new(database.clone())),
                Arc::new(battlenet::BattleNetPlatform::new(
                    database.clone(),
                    app_handle.clone(),
                )),
                Arc::new(epic::EpicPlatform::new(database.clone())),
            ],
            steam_platform,
            database,
            app_handle,
            game_monitor,
            metadata_service: Arc::new(metadata_service),
        })
    }

    pub async fn initialize_db(&self) -> Result<(), AppError> {
        // Initialiser la base de données ici
        self.database.initialize().await
    }

    pub async fn scan_all_platforms(&self, use_cache: bool) -> GameResult<Vec<Game>> {
        if use_cache {
            let games = self.database.games().get_all_games().await?;
            return Ok(games);
        }

        let mut found_game_ids = HashSet::new();
        let mut new_or_updated_games = Vec::new();

        for platform in &self.platforms {
            let platform_name = platform.platform_name();

            match platform.scan_games().await {
                Ok(scanned_games) => {
                    for scanned_game in scanned_games {
                        found_game_ids.insert(scanned_game.id.clone());

                        // Vérifier si le jeu existe déjà
                        if let Ok(Some(existing_game)) = self.database.games().get_game(&scanned_game.id).await {
                            // Ne mettre à jour que les informations d'installation si nécessaire
                            if existing_game.installation.version != scanned_game.installation.version
                                || existing_game.installation.install_path != scanned_game.installation.install_path {

                                // Créer un nouveau jeu en préservant les métadonnées existantes
                                let updated_game = Game {
                                    installation: scanned_game.installation,
                                    // Conserver les métadonnées et médias existants
                                    metadata: existing_game.metadata,
                                    media: existing_game.media,
                                    // Garder les autres informations existantes
                                    stats: existing_game.stats,
                                    last_played: existing_game.last_played,
                                    ..scanned_game
                                };

                                self.database.games().upsert_game(&updated_game).await?;
                                new_or_updated_games.push(updated_game);
                            } else {
                                new_or_updated_games.push(existing_game);
                            }
                        } else {
                            // Nouveau jeu, l'ajouter tel quel
                            self.database.games().upsert_game(&scanned_game).await?;
                            new_or_updated_games.push(scanned_game);
                        }
                    }
                }
                Err(e) => eprintln!("Error scanning {}: {}", platform_name, e),
            }
        }

        // Nettoyer uniquement les jeux qui n'existent plus (désinstallés)
        // en excluant toujours les jeux customs
        let all_games = self.database.games().get_all_games().await?;
        for game in all_games {
            if game.platform != Platform::Custom
                && !found_game_ids.contains(&game.id)
                && !game.installation.install_path.exists() {
                self.database.games().delete_game(&game.id).await?;
            }
        }

        Ok(new_or_updated_games)
    }

    async fn clean_missing_games(&self, found_game_ids: &HashSet<String>) -> Result<(), AppError> {
        let all_games = self.database.games().get_all_games().await?;

        for game in all_games {
            // Ne pas supprimer les jeux customs
            if game.platform != Platform::Custom && !found_game_ids.contains(&game.id) {
                self.database.games().delete_game(&game.id).await?;
            }
        }

        Ok(())
    }

    pub async fn launch_game(&self, game_id: &str) -> GameResult<()> {
        // Récupérer le jeu pour le monitoring
        if let Some(game) = self.get_game(game_id).await? {
            match game.platform {
                Platform::Custom => {
                    // Pour les jeux custom, lancer directement l'exécutable
                    if let Some(executable) = game.installation.executable.as_ref() {
                        Command::new(executable)
                            .spawn()
                            .map_err(|e| AppError {
                                message: format!("Failed to launch custom game: {}", e),
                            })?;

                        // Utiliser le même système de retry que pour les autres jeux
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        for _ in 0..5 {
                            if let Some(session) = self.game_monitor.track_game(&game).await {
                                return Ok(());
                            }
                            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        }
                    }
                    Err(AppError {
                        message: "No executable path found for custom game".to_string(),
                    })
                },
                _ => {
                    // Garder le code existant pour les autres plateformes
                    for platform in &self.platforms {
                        if let Ok(()) = platform.launch_game(game_id).await {
                            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                            for _ in 0..5 {
                                if let Some(session) = self.game_monitor.track_game(&game).await {
                                    return Ok(());
                                }
                                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                            }
                        }
                    }
                    Err(AppError {
                        message: "Game not found or couldn't be launched".to_string(),
                    })
                }
            }
        } else {
            Err(AppError {
                message: "Game not found".to_string(),
            })
        }
    }

    pub async fn update_game_metadata(&self, game_id: &str) -> GameResult<()> {
        if let Some(mut game) = self.get_game(game_id).await? {
            self.metadata_service.update_metadata(&mut game).await?;
            // Mettre à jour le jeu en base
            self.database.games().upsert_game(&game).await?;
        }
        Ok(())
    }

    pub async fn get_game(&self, game_id: &str) -> GameResult<Option<Game>> {
        self.database.games().get_game(game_id).await
    }

    pub async fn add_custom_game(&self, config: CustomGameConfig) -> GameResult<Game> {
        // Générer un ID unique pour le jeu custom
        let custom_id = format!("custom_{}", uuid::Uuid::new_v4());

        // Vérifier si l'exécutable existe
        let executable_path = PathBuf::from(&config.executable_path);
        if !executable_path.exists() {
            return Err(AppError {
                message: format!("Executable not found: {}", config.executable_path)
            });
        }

        // Vérifier si le dossier d'installation existe
        let install_path = PathBuf::from(&config.install_path);
        if !install_path.exists() {
            return Err(AppError {
                message: format!("Installation path not found: {}", config.install_path)
            });
        }

        let icon_path = config.icon_path; // Le move se fait ici
        let game = Game {
           id: custom_id.clone(),
           platform_id: custom_id,
           platform: Platform::Custom,
           title: config.title.clone(),
           installation: GameInstallation {
               install_path: install_path.clone(),
               executable: Some(config.executable_path),
               size: Self::calculate_folder_size(&install_path)?,
               version: None,
               last_updated: None,
           },
           metadata: GameMetadata {
               title: config.title,
               description: None,
               developer: None,
               publisher: None,
               release_date: None,
               genres: Vec::new(),
               tags: Vec::new(),
               media: None,
           },
           media: GameMedia {
               thumbnail: icon_path.clone(), // Utiliser le clone
               cover: None,
               screenshots: Vec::new(),
               background: None,
               icon: icon_path,  // Utiliser la valeur originale
               logo: None,
           },
           last_played: None,
           stats: GameStats {
               total_playtime: 0,
               last_session_duration: 0,
               sessions_count: 0,
               first_played: None,
               last_played: None,
           },
        };
        // Sauvegarder le jeu dans la base de données
        self.database.games().upsert_game(&game).await?;

        Ok(game)
    }

    fn calculate_folder_size(path: &Path) -> GameResult<u64> {
        let mut total_size = 0u64;
        for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.path().is_file() {
                total_size += entry.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
        Ok(total_size)
    }

    pub async fn delete_game(&self, game_id: &str) -> GameResult<()> {
        self.database.games().delete_game(game_id).await
    }
}

// Implement Send + Sync
unsafe impl Send for GameManager {}
unsafe impl Sync for GameManager {}

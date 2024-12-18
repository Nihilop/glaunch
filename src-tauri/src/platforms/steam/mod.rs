mod launcher;
mod scanner;

use crate::models::{Game, GameResult};
use crate::platforms::traits::*;
use crate::utils::AppError;
use crate::Database;
use std::path::PathBuf;
use std::sync::Arc;

pub use launcher::SteamGameLauncher;
pub use scanner::SteamGameScanner;

pub struct SteamPlatform {
    scanner: Arc<SteamGameScanner>,
    launcher: Arc<SteamGameLauncher>,
}

impl SteamPlatform {
    pub fn new(database: Arc<Database>) -> Self {
        Self {
            scanner: Arc::new(SteamGameScanner::new()),
            launcher: Arc::new(SteamGameLauncher::new()),
        }
    }
    pub async fn get_game_by_id(&self, platform_specific_id: &str) -> GameResult<Option<Game>> {
        // Cette méthode pourrait être optimisée pour chercher directement le jeu
        // au lieu de scanner toute la bibliothèque
        let game_path = format!("appmanifest_{}.acf", platform_specific_id);
        for library in self.scanner.get_library_paths() {
            let full_path = library.join(&game_path);
            if full_path.exists() {
                if let Ok(game) = self.scanner.parse_acf_file(&full_path).await {
                    return Ok(Some(game));
                }
            }
        }
        Ok(None)
    }
}

#[async_trait::async_trait]
impl GamePlatform for SteamPlatform {
    fn platform_name(&self) -> &'static str {
        "Steam"
    }

    fn supported_file_types(&self) -> Vec<&'static str> {
        vec!["acf"]
    }

    async fn initialize(&mut self) -> GameResult<()> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl GameScanner for SteamPlatform {
    async fn scan_games(&self) -> GameResult<Vec<Game>> {
        self.scanner.scan_games().await
    }

    fn get_library_paths(&self) -> Vec<PathBuf> {
        self.scanner.get_library_paths()
    }

    async fn add_library_path(&mut self, path: PathBuf) -> GameResult<()> {
        if let Some(scanner) = Arc::get_mut(&mut self.scanner) {
            scanner.add_library_path(path).await
        } else {
            Err(AppError {
                message: "Failed to get mutable reference to scanner".into(),
            })
        }
    }
}

#[async_trait::async_trait]
impl GameLauncher for SteamPlatform {
    async fn launch_game(&self, game_id: &str) -> GameResult<()> {
        self.launcher.launch_game(game_id).await
    }

    async fn stop_game(&self, game_id: &str) -> GameResult<()> {
        self.launcher.stop_game(game_id).await
    }

    async fn is_game_running(&self, game_id: &str) -> GameResult<bool> {
        self.launcher.is_game_running(game_id).await
    }
}

#[async_trait::async_trait]
impl MetadataProvider for SteamPlatform {
    async fn update_metadata(&self, _game: &mut Game) -> GameResult<()> {
        // La mise à jour des métadonnées est maintenant gérée par le service RAWG
        Ok(())
    }
}

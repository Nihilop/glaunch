mod launcher;
mod scanner;
use crate::models::{Game, GameResult};
use crate::platforms::traits::*;
use crate::Database;
use std::path::PathBuf;
use std::sync::Arc;

pub use launcher::EpicGameLauncher;
pub use scanner::EpicGameScanner;

pub struct EpicPlatform {
    scanner: EpicGameScanner,
    launcher: EpicGameLauncher,
    database: Arc<Database>,
}

impl EpicPlatform {
    pub fn new(database: Arc<Database>) -> Self {
        Self {
            scanner: EpicGameScanner::new(),
            launcher: EpicGameLauncher::new(),
            database,
        }
    }
}

#[async_trait::async_trait]
impl GamePlatform for EpicPlatform {
    fn platform_name(&self) -> &'static str {
        "Epic Games"
    }

    fn supported_file_types(&self) -> Vec<&'static str> {
        vec![] // Epic utilise un format de manifeste différent
    }

    async fn initialize(&mut self) -> GameResult<()> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl GameScanner for EpicPlatform {
    async fn scan_games(&self) -> GameResult<Vec<Game>> {
        self.scanner.scan_games().await
    }

    fn get_library_paths(&self) -> Vec<PathBuf> {
        vec![self.scanner.get_install_path()]
    }

    async fn add_library_path(&mut self, path: PathBuf) -> GameResult<()> {
        self.scanner.add_user_folder(path)
    }
}

#[async_trait::async_trait]
impl GameLauncher for EpicPlatform {
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
impl MetadataProvider for EpicPlatform {}

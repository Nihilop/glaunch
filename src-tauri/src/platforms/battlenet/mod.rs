// Dans platforms/battlenet/mod.rs
mod launcher;
mod scanner;
use crate::models::{Game, GameResult};
use crate::platforms::traits::*;
use crate::Database;
use async_trait::async_trait;
pub use launcher::BattleNetGameLauncher;
pub use scanner::BattleNetGameScanner;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::AppHandle;

pub struct BattleNetPlatform {
    scanner: BattleNetGameScanner,
    launcher: BattleNetGameLauncher,
    database: Arc<Database>,
}

impl BattleNetPlatform {
    pub fn new(database: Arc<Database>, app_handle: AppHandle) -> Self {
        Self {
            scanner: BattleNetGameScanner::new(),
            launcher: BattleNetGameLauncher::new(app_handle),
            database,
        }
    }
}

#[async_trait]
impl GamePlatform for BattleNetPlatform {
    fn platform_name(&self) -> &'static str {
        "Battle.net"
    }

    fn supported_file_types(&self) -> Vec<&'static str> {
        vec![] // Battle.net n'a pas de types de fichiers spécifiques à scanner
    }

    async fn initialize(&mut self) -> GameResult<()> {
        Ok(()) // Rien de spécial à initialiser pour Battle.net
    }
}

#[async_trait]
impl GameScanner for BattleNetPlatform {
    async fn scan_games(&self) -> GameResult<Vec<Game>> {
        self.scanner.scan_games().await
    }

    fn get_library_paths(&self) -> Vec<PathBuf> {
        vec![] // Les chemins sont gérés via le fichier de config
    }

    async fn add_library_path(&mut self, _path: PathBuf) -> GameResult<()> {
        Ok(()) // Battle.net gère ses propres chemins
    }
}

#[async_trait]
impl GameLauncher for BattleNetPlatform {
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

#[async_trait]
impl MetadataProvider for BattleNetPlatform {}

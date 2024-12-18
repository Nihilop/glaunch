mod launcher;

use crate::models::{Game, GameResult};
use crate::platforms::traits::*;
use crate::Database;
use std::path::PathBuf;
use std::sync::Arc;

pub use launcher::CustomGameLauncher;

pub struct CustomPlatform {
    launcher: CustomGameLauncher,
    database: Arc<Database>,
}

impl CustomPlatform {
    pub fn new(database: Arc<Database>) -> Self {
        Self {
            launcher: CustomGameLauncher::new(),
            database,
        }
    }
}

#[async_trait::async_trait]
impl GamePlatform for CustomPlatform {
    fn platform_name(&self) -> &'static str {
        "Custom"
    }

    fn supported_file_types(&self) -> Vec<&'static str> {
        vec!["exe"]
    }

    async fn initialize(&mut self) -> GameResult<()> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl GameLauncher for CustomPlatform {
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
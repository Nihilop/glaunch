use crate::models::{Game, GameResult};
use async_trait::async_trait;
use std::path::PathBuf;

#[async_trait]
pub trait GameScanner: Send + Sync {
    async fn scan_games(&self) -> GameResult<Vec<Game>>;
    fn get_library_paths(&self) -> Vec<PathBuf>;
    async fn add_library_path(&mut self, path: PathBuf) -> GameResult<()>;
}

#[async_trait]
pub trait GameLauncher: Send + Sync {
    async fn launch_game(&self, game_id: &str) -> GameResult<()>;
    async fn stop_game(&self, game_id: &str) -> GameResult<()>;
    async fn is_game_running(&self, game_id: &str) -> GameResult<bool>;
}

#[async_trait]
pub trait MetadataProvider: Send + Sync {
    async fn update_metadata(&self, game: &mut Game) -> GameResult<()> {
        Ok(())
    }
}

#[async_trait]
pub trait GamePlatform: GameScanner + GameLauncher + MetadataProvider {
    fn platform_name(&self) -> &'static str;
    fn supported_file_types(&self) -> Vec<&'static str>;
    async fn initialize(&mut self) -> GameResult<()>;
}

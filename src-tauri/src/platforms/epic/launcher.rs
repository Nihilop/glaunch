use crate::models::GameResult;
use crate::utils::AppError;
use std::process::Command;

pub struct EpicGameLauncher;

impl EpicGameLauncher {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl crate::platforms::traits::GameLauncher for EpicGameLauncher {
    async fn launch_game(&self, game_id: &str) -> GameResult<()> {
        let platform_id = game_id.strip_prefix("epic_").ok_or_else(|| AppError {
            message: "Invalid Epic game ID format".to_string(),
        })?;

        let launch_url = format!(
            "com.epicgames.launcher://apps/{}?action=launch",
            platform_id
        );

        Command::new("cmd")
            .args(&["/C", "start", "", &launch_url])
            .spawn()
            .map_err(|e| AppError {
                message: format!("Failed to launch Epic game: {}", e),
            })?;

        Ok(())
    }

    async fn stop_game(&self, _game_id: &str) -> GameResult<()> {
        // Epic Games n'a pas d'API directe pour arrêter les jeux
        Ok(())
    }

    async fn is_game_running(&self, _game_id: &str) -> GameResult<bool> {
        // TODO: Implémenter la vérification via le processus du jeu
        Ok(false)
    }
}

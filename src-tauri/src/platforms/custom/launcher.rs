// Dans platforms/custom/launcher.rs (à créer)
use crate::models::GameResult;
use crate::utils::AppError;
use std::process::Command;

pub struct CustomGameLauncher;

#[async_trait::async_trait]
impl crate::platforms::traits::GameLauncher for CustomGameLauncher {
    async fn launch_game(&self, game_id: &str) -> GameResult<()> {
        if let Some(executable) = game.installation.executable.as_ref() {
            Command::new(executable)
                .spawn()
                .map_err(|e| AppError {
                    message: format!("Failed to launch custom game: {}", e),
                })?;
            Ok(())
        } else {
            Err(AppError {
                message: "No executable path found for custom game".to_string(),
            })
        }
    }

    async fn stop_game(&self, _game_id: &str) -> GameResult<()> {
        // Pour les jeux custom, on ne gère pas l'arrêt
        Ok(())
    }

    async fn is_game_running(&self, _game_id: &str) -> GameResult<bool> {
        // On pourrait implémenter une vérification basique du processus
        Ok(false)
    }
}
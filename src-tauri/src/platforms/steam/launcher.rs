use crate::models::GameResult;
use crate::utils::AppError;
use std::process::Command;

pub struct SteamGameLauncher {
    steam_path: std::path::PathBuf,
}

impl SteamGameLauncher {
    pub fn new() -> Self {
        Self {
            steam_path: std::path::PathBuf::from("C:\\Program Files (x86)\\Steam\\steam.exe"),
        }
    }

    async fn ensure_steam_running(&self) -> GameResult<()> {
        if !self.check_steam_process().await? {
            self.start_steam()?;
            // Attendre que Steam démarre
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
        Ok(())
    }

    async fn check_steam_process(&self) -> GameResult<bool> {
        let output = std::process::Command::new("tasklist")
            .output()
            .map_err(|e| AppError {
                message: format!("Failed to check processes: {}", e),
            })?;

        let processes = String::from_utf8_lossy(&output.stdout);
        Ok(processes.contains("steam.exe"))
    }

    fn start_steam(&self) -> GameResult<()> {
        if !self.steam_path.exists() {
            return Err(AppError {
                message: "Steam executable not found".to_string(),
            });
        }

        Command::new(&self.steam_path)
            .spawn()
            .map_err(|e| AppError {
                message: format!("Failed to start Steam: {}", e),
            })?;

        Ok(())
    }

    fn extract_app_id(&self, game_id: &str) -> GameResult<String> {
        let parts: Vec<&str> = game_id.split('_').collect();
        if parts.len() != 2 || parts[0] != "steam" {
            return Err(AppError {
                message: format!("Invalid Steam game ID format: {}", game_id),
            });
        }
        Ok(parts[1].to_string())
    }
}

#[async_trait::async_trait]
impl crate::platforms::traits::GameLauncher for SteamGameLauncher {
    async fn launch_game(&self, game_id: &str) -> GameResult<()> {
        self.ensure_steam_running().await?;

        let app_id = self.extract_app_id(game_id)?;

        let status = Command::new("cmd")
            .args(&["/C", &format!("start steam://run/{}", app_id)])
            .status()
            .map_err(|e| AppError {
                message: format!("Failed to launch Steam game: {}", e),
            })?;

        if !status.success() {
            return Err(AppError {
                message: format!("Failed to launch game with app_id: {}", app_id),
            });
        }

        Ok(())
    }

    async fn stop_game(&self, game_id: &str) -> GameResult<()> {
        let app_id = self.extract_app_id(game_id)?;

        // Utiliser l'URI Steam pour arrêter le jeu
        let status = Command::new("cmd")
            .args(&["/C", &format!("start steam://stop/{}", app_id)])
            .status()
            .map_err(|e| AppError {
                message: format!("Failed to stop Steam game: {}", e),
            })?;

        if !status.success() {
            return Err(AppError {
                message: format!("Failed to stop game with app_id: {}", app_id),
            });
        }

        Ok(())
    }

    async fn is_game_running(&self, game_id: &str) -> GameResult<bool> {
        let app_id = self.extract_app_id(game_id)?;

        let output = Command::new("tasklist").output().map_err(|e| AppError {
            message: format!("Failed to check running processes: {}", e),
        })?;

        let processes = String::from_utf8_lossy(&output.stdout);

        // Cette vérification est basique et pourrait être améliorée
        Ok(processes.contains(&format!("steam_app_{}.exe", app_id)))
    }
}

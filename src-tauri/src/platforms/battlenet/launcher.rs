// Dans platforms/battlenet/launcher.rs
use crate::models::GameResult;
use crate::platforms::battlenet::scanner::BattleNetGameScanner;
use crate::platforms::traits::GameLauncher;
use crate::utils::AppError;
use async_trait::async_trait;
use std::process::Command;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;

#[derive(Debug)] // Optionnel mais utile pour le debug
pub struct BattleNetGameLauncher {
    scanner: BattleNetGameScanner,
    app_handle: AppHandle,
}

impl BattleNetGameLauncher {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            scanner: BattleNetGameScanner::new(),
            app_handle,
        }
    }

    fn get_protocol_uri(&self, game_id: &str) -> Option<&'static str> {
        match game_id.split('_').nth(1)? {
            "wow" => Some("battlenet://WoW"),
            "prometheus" => Some("battlenet://DIABLO4"),
            "d3" => Some("battlenet://D3"),
            "ow" => Some("battlenet://OVERWATCH"),
            "hs" => Some("battlenet://HEARTHSTONE"),
            _ => None,
        }
    }
}

fn get_launch_code(game_id: &str) -> Option<&'static str> {
    // Extraire l'ID spécifique après le préfixe "battlenet_"
    match game_id.strip_prefix("battlenet_")? {
        "wow" => Some("WoW"),            // World of Warcraft
        "ow" => Some("Pro"),             // Overwatch 2
        "prometheus" => Some("DIABLO4"), // Diablo IV
        "d3" => Some("D3"),              // Diablo III
        "hs" => Some("WTCG"),            // Hearthstone
        "hots" => Some("HOTS"),          // Heroes of the Storm
        _ => None,
    }
}

#[async_trait]
impl GameLauncher for BattleNetGameLauncher {
    async fn launch_game(&self, game_id: &str) -> GameResult<()> {
        if let Some(launch_code) = get_launch_code(game_id) {
            // Commande exacte comme dans le raccourci
            let battlenet_exe = r"C:\Program Files (x86)\Battle.net\Battle.net.exe";
            let launch_arg = format!("--exec=launch {}", launch_code);
            let shell = self.app_handle.shell();

            let output = shell
                .command(battlenet_exe)
                .args([&launch_arg])
                .output()
                .await
                .unwrap();

            if output.status.success() {
                Ok(())
            } else {
                Err(AppError {
                    message: format!("Command failed with exit code: {:?}", output.status.code()),
                })
            }
        } else {
            Err(AppError {
                message: "Invalid Battle.net game ID or unsupported game".to_string(),
            })
        }
    }

    async fn stop_game(&self, _game_id: &str) -> GameResult<()> {
        // Battle.net ne fournit pas de moyen direct d'arrêter les jeux
        Ok(())
    }

    async fn is_game_running(&self, game_id: &str) -> GameResult<bool> {
        if let Some(exe_name) = game_id.split('_').nth(1).and_then(|id| match id {
            "wow" => Some("Wow.exe"),
            "prometheus" => Some("Diablo IV.exe"),
            "d3" => Some("Diablo III.exe"),
            "ow" => Some("Overwatch.exe"),
            "hs" => Some("Hearthstone.exe"),
            _ => None,
        }) {
            let output = Command::new("tasklist").output().map_err(|e| AppError {
                message: format!("Failed to check running processes: {}", e),
            })?;

            let processes = String::from_utf8_lossy(&output.stdout);
            Ok(processes.contains(exe_name))
        } else {
            Ok(false)
        }
    }
}

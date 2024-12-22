use crate::models::GameResult;
use crate::platforms::battlenet::scanner::BattleNetGameScanner;
use crate::platforms::traits::GameLauncher;
use crate::utils::AppError;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use winreg::enums::*;
use winreg::RegKey;

#[derive(Debug)]
struct GameLaunchInfo {
    game_id: String,
    launch_code: &'static str,
    process_name: &'static str,
    process_name_alt: Option<&'static str>,
    wait_time: Duration,
}

#[derive(Debug)]
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

    fn get_launch_info(game_id: &str) -> Option<GameLaunchInfo> {
        match game_id.strip_prefix("battlenet_")? {
            "wow" => Some(GameLaunchInfo {
                game_id: "wow".to_string(),
                launch_code: "WoW",
                process_name: "Wow.exe",
                process_name_alt: Some("Wow.exe"),
                wait_time: Duration::from_secs(5),
            }),
            "prometheus" => Some(GameLaunchInfo {
                game_id: "overwatch".to_string(),
                launch_code: "Pro",
                process_name: "Overwatch Application.exe",
                process_name_alt: Some("Overwatch.exe"),
                wait_time: Duration::from_secs(3),
            }),
            "fenris" => Some(GameLaunchInfo {
                game_id: "diablo4".to_string(),
                launch_code: "OSI",
                process_name: "Diablo IV.exe",
                process_name_alt: Some("Diablo.exe"),
                wait_time: Duration::from_secs(5),
            }),
            "d3" => Some(GameLaunchInfo {
                game_id: "d3".to_string(),
                launch_code: "D3",
                process_name: "Diablo III.exe",
                process_name_alt: Some("Diablo III.exe"),
                wait_time: Duration::from_secs(3),
            }),
            "hs" => Some(GameLaunchInfo {
                game_id: "hearthstone".to_string(),
                launch_code: "WTCG",
                process_name: "Hearthstone.exe",
                process_name_alt: Some("Hearthstone.exe"),
                wait_time: Duration::from_secs(3),
            }),
            _ => None,
        }
    }

    fn get_battlenet_path() -> Result<PathBuf, AppError> {
        // Recherche dans les emplacements standards
        let standard_paths = [
            r"C:\Program Files (x86)\Battle.net\Battle.net.exe",
            r"C:\Program Files\Battle.net\Battle.net.exe",
        ];

        for path in &standard_paths {
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(path);
            }
        }

        // Si non trouvé, chercher dans le registre
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let key = hklm
            .open_subkey(r"SOFTWARE\WOW6432Node\Battle.net\Launch")
            .or_else(|_| hklm.open_subkey(r"SOFTWARE\Battle.net\Launch"))
            .map_err(|e| AppError {
                message: format!("Failed to find Battle.net in registry: {}", e),
            })?;

        let path: String = key.get_value("InstallPath").map_err(|e| AppError {
            message: format!("Failed to get Battle.net path from registry: {}", e),
        })?;

        let launcher_path = PathBuf::from(path).join("Battle.net.exe");
        if launcher_path.exists() {
            Ok(launcher_path)
        } else {
            Err(AppError {
                message: "Battle.net launcher not found".to_string(),
            })
        }
    }

    async fn ensure_battlenet_running(&self) -> Result<(), AppError> {
        // Vérifier si Battle.net est déjà en cours d'exécution
        let shell = self.app_handle.shell();
        let output = shell
            .command("tasklist")
            .output()
            .await
            .map_err(|e| AppError {
                message: format!("Failed to check running processes: {}", e),
            })?;

        let process_list = String::from_utf8_lossy(&output.stdout);
        if !process_list.contains("Battle.net.exe") {
            // Lancer Battle.net si non en cours d'exécution
            let launcher_path = Self::get_battlenet_path()?;
            shell
                .command(launcher_path)
                .spawn()
                .map_err(|e| AppError {
                    message: format!("Failed to start Battle.net: {}", e),
                })?;

            // Attendre que Battle.net démarre
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
        Ok(())
    }

    async fn wait_for_game_process(&self, launch_info: &GameLaunchInfo, timeout: Duration) -> Result<bool, AppError> {
        let start_time = std::time::Instant::now();

        while start_time.elapsed() < timeout {
            let shell = self.app_handle.shell();
            let output = shell
                .command("tasklist")
                .output()
                .await
                .map_err(|e| AppError {
                    message: format!("Failed to check processes: {}", e),
                })?;

            let processes = String::from_utf8_lossy(&output.stdout);

            // Vérifier le processus principal
            if processes.contains(launch_info.process_name) {
                return Ok(true);
            }

            // Vérifier le processus alternatif si présent
            if let Some(alt_process) = launch_info.process_name_alt {
                if processes.contains(alt_process) {
                    return Ok(true);
                }
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        Ok(false)
    }
}

#[async_trait::async_trait]
impl GameLauncher for BattleNetGameLauncher {
    async fn launch_game(&self, game_id: &str) -> GameResult<()> {
        // Obtenir les informations de lancement
        let launch_info = Self::get_launch_info(game_id).ok_or_else(|| AppError {
            message: format!("Unsupported Battle.net game: {}", game_id),
        })?;

        // S'assurer que Battle.net est en cours d'exécution
        self.ensure_battlenet_running().await?;

        // Obtenir le chemin du launcher Battle.net
        let launcher_path = Self::get_battlenet_path()?;

        // Lancer le jeu
        let shell = self.app_handle.shell();
        shell
            .command(&launcher_path)
            .args(&["--exec", &format!("launch {}", launch_info.launch_code)])
            .spawn()
            .map_err(|e| AppError {
                message: format!("Failed to launch game: {}", e),
            })?;

        // Attendre que le processus du jeu démarre
        if !self
            .wait_for_game_process(&launch_info, Duration::from_secs(30))
            .await?
        {
            return Err(AppError {
                message: format!("Game process {} did not start", launch_info.process_name),
            });
        }

        Ok(())
    }

    async fn stop_game(&self, _game_id: &str) -> GameResult<()> {
        // Battle.net ne fournit pas de moyen direct d'arrêter les jeux
        Ok(())
    }

    async fn is_game_running(&self, game_id: &str) -> GameResult<bool> {
        if let Some(launch_info) = Self::get_launch_info(game_id) {
            let shell = self.app_handle.shell();
            let output = shell
                .command("tasklist")
                .output()
                .await
                .map_err(|e| AppError {
                    message: format!("Failed to check running processes: {}", e),
                })?;

            let processes = String::from_utf8_lossy(&output.stdout);
            Ok(processes.contains(launch_info.process_name))
        } else {
            Ok(false)
        }
    }
}
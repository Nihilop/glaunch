use crate::models::{
    Game, GameInstallation, GameMedia, GameMetadata, GameResult, GameStats, Platform,
};
use crate::utils::AppError;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::log_debug;

#[derive(Debug, Deserialize)]
struct BnetConfig {
    Client: ClientConfig,
    #[serde(rename = "Games")]
    Games: HashMap<String, GameConfig>,
    #[serde(flatten)]
    servers: HashMap<String, ServerInfo>,
}

#[derive(Debug, Deserialize)]
struct ClientConfig {
    Install: Option<InstallConfig>,
    Version: Option<VersionConfig>,
}

#[derive(Debug, Deserialize)]
struct InstallConfig {
    DefaultInstallPath: String,
}

#[derive(Debug, Deserialize)]
struct VersionConfig {
    FirstRun: Option<String>,
    Release: Option<ReleaseConfig>,
}

#[derive(Debug, Deserialize)]
struct ReleaseConfig {
    FirstRun: Option<String>,
    LastBuildVersion: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GameConfig {
    ServerUid: String,
    LastPlayed: Option<String>,
    Resumable: String,
    AutoUpdateCriteriaPassed: Option<String>,
    LastActioned: Option<String>,
    AdditionalLaunchArguments: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ServerInfo {
    Path: Option<String>,
    Client: Option<ServerClientConfig>,
    Services: Option<ServicesConfig>,
}

#[derive(Debug, Deserialize)]
struct ServerClientConfig {
    Language: Option<String>,
    LoginSettings: Option<LoginSettings>,
}

#[derive(Debug, Deserialize)]
struct LoginSettings {
    AllowedRegions: String,
    AllowedLocales: String,
}

#[derive(Debug, Deserialize)]
struct ServicesConfig {
    LastLoginRegion: String,
    LastLoginAddress: String,
    LastLoginTassadar: String,
}

#[derive(Debug, Deserialize)]
struct InstallInfo {
    path: Option<String>,
}

#[derive(Debug)]
struct BnetGameInfo {
    name: &'static str,
    game_dir: &'static str,
    launcher_exe: &'static str,
    launch_code: &'static str,
    game_exe: &'static str,
    game_subdir: &'static str,
}

#[derive(Debug)]
pub struct BattleNetGameScanner {
    known_games: HashMap<String, BnetGameInfo>,
    bnet_launcher_path: Option<PathBuf>,
}

impl BattleNetGameScanner {
    pub fn new() -> Self {
        let mut known_games = HashMap::new();

        // Liste des jeux Battle.net connus
        known_games.insert(
            "wow".to_string(),
            BnetGameInfo {
                name: "World of Warcraft",
                game_dir: "World of Warcraft",
                launcher_exe: "World of Warcraft Launcher.exe",
                launch_code: "WoW",
                game_exe: "Wow.exe",
                game_subdir: "_retail_",
            },
        );

        known_games.insert(
            "prometheus".to_string(),
            BnetGameInfo {
                name: "Overwatch 2",
                game_dir: "Overwatch",
                launcher_exe: "Overwatch Launcher.exe",
                launch_code: "Pro",
                game_exe: "Overwatch.exe",
                game_subdir: "_retail_",
            },
        );

        known_games.insert(
            "d4".to_string(),
            BnetGameInfo {
                name: "Diablo IV",
                game_dir: "Diablo IV",
                launcher_exe: "Diablo IV Launcher.exe",
                launch_code: "D3",
                game_exe: "Diablo IV.exe",
                game_subdir: "_retail_",
            },
        );

        known_games.insert(
            "d3".to_string(),
            BnetGameInfo {
                name: "Diablo III",
                game_dir: "Diablo III",
                launcher_exe: "Diablo III Launcher.exe",
                launch_code: "Pro", // rename
                game_exe: "Diablo III.exe",
                game_subdir: "_retail_",
            },
        );

        known_games.insert(
            "hs".to_string(),
            BnetGameInfo {
                name: "Hearthstone",
                game_dir: "Hearthstone",
                launcher_exe: "Hearthstone Launcher.exe",
                launch_code: "Pro", // rename
                game_exe: "Hearthstone.exe",
                game_subdir: "_retail_",
            },
        );

        known_games.insert(
            "hots".to_string(),
            BnetGameInfo {
                name: "Heroes of the Storm",
                game_dir: "Heroes of the Storm",
                launcher_exe: "Heroes of the Storm Launcher.exe",
                launch_code: "Pro", // rename
                game_exe: "Heroes of the Storm.exe",
                game_subdir: "_retail_",
            },
        );

        known_games.insert(
            "scr".to_string(),
            BnetGameInfo {
                name: "StarCraft: Remastered",
                game_dir: "StarCraft",
                launcher_exe: "StarCraft Launcher.exe",
                launch_code: "Pro", // rename
                game_exe: "StarCraft.exe",
                game_subdir: "_retail_",
            },
        );

        known_games.insert(
            "s2".to_string(),
            BnetGameInfo {
                name: "StarCraft II",
                game_dir: "StarCraft II",
                launcher_exe: "StarCraft II Launcher.exe",
                launch_code: "Pro", // rename
                game_exe: "StarCraft II.exe",
                game_subdir: "_retail_",
            },
        );

        known_games.insert(
            "w3".to_string(),
            BnetGameInfo {
                name: "Warcraft III: Reforged",
                game_dir: "Warcraft III",
                launcher_exe: "Warcraft III Launcher.exe",
                launch_code: "Pro", // rename
                game_exe: "Warcraft III.exe",
                game_subdir: "_retail_",
            },
        );

        known_games.insert(
            "viper".to_string(),
            BnetGameInfo {
                name: "Call of Duty: Modern Warfare",
                game_dir: "Call of Duty",
                launcher_exe: "Call of Duty Launcher.exe",
                launch_code: "Pro", // rename
                game_exe: "ModernWarfare.exe",
                game_subdir: "_retail_",
            },
        );

        Self {
            known_games,
            bnet_launcher_path: None,
        }
    }

    fn get_config_path() -> GameResult<PathBuf> {
        let app_data = std::env::var("APPDATA").map_err(|_| AppError {
            message: "Failed to get APPDATA path".to_string(),
        })?;

        Ok(PathBuf::from(app_data)
            .join("Battle.net")
            .join("Battle.net.config"))
    }

    fn calculate_folder_size(&self, path: &Path) -> GameResult<u64> {
        let mut total_size = 0;
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                }
            }
        }
        Ok(total_size)
    }

    fn find_battlenet_launcher(&self, base_paths: &[PathBuf]) -> GameResult<PathBuf> {
        for base_path in base_paths {
            let launcher_path = base_path.join("Battle.net").join("Battle.net.exe");
            if launcher_path.exists() {
                return Ok(launcher_path);
            }
        }
        Err(AppError {
            message: "Could not find Battle.net launcher".to_string(),
        })
    }

    fn create_launch_command(launcher_path: &Path, launch_code: &str) -> String {
        // Convertir le path en string et remplacer les doubles backslashes par des simples
        let launcher_str = launcher_path.to_string_lossy().replace("\\", "/");

        format!(
            "cmd /c start \"\" \"{launcher_str}\" --exec=\"launch {launch_code}\"",
            launcher_str = launcher_str,
            launch_code = launch_code
        )
    }

    pub async fn scan_games(&self) -> GameResult<Vec<Game>> {
        log_debug!("🔍 Starting Battle.net games scan...");

        let base_paths = vec![
            PathBuf::from("C:\\Program Files (x86)"),
            PathBuf::from("C:\\Program Files"),
        ];

        let bnet_launcher_path = self.find_battlenet_launcher(&base_paths)?;
        log_debug!("🎮 Found Battle.net launcher at: {:?}", bnet_launcher_path);

        let config_path = Self::get_config_path()?;
        log_debug!("📄 Reading config from: {:?}", config_path);

        let config_content = fs::read_to_string(&config_path).map_err(|e| AppError {
            message: format!("Error reading Battle.net.config: {}", e),
        })?;

        let config: BnetConfig = serde_json::from_str(&config_content).map_err(|e| AppError {
            message: format!("Error parsing Battle.net.config: {}", e),
        })?;

        let mut games = Vec::new();
        let mut paths_to_check = Vec::new();

        // 1. Chemin d'installation par défaut s'il existe
        if let Some(default_path) = config.Client.Install.as_ref().map(|install| PathBuf::from(&install.DefaultInstallPath)) {
            paths_to_check.push(default_path);
        }

        // 2. Vérifier les chemins dans la config, y compris sous les tokens aléatoires
        let mut found_battlenet_path = false;
        for (token, server_info) in &config.servers {
            if let Some(path) = &server_info.Path {
                let path_buf = PathBuf::from(path);
                if path_buf.ends_with("Battle.net") {
                    log_debug!("Found Battle.net path under token {}: {:?}", token, path_buf);
                    found_battlenet_path = true;
                    // Si on trouve le dossier Battle.net, on l'ajoute en premier dans la liste
                    paths_to_check.insert(0, path_buf.clone());

                    // On ajoute aussi le dossier parent qui pourrait contenir d'autres jeux
                    if let Some(parent) = path_buf.parent() {
                        paths_to_check.push(parent.to_path_buf());
                    }
                }
            }
        }

        // 3. Ajouter les chemins standards si on n'a pas trouvé de chemin Battle.net explicite
        if !found_battlenet_path {
            let standard_paths = vec![
                PathBuf::from("C:\\Program Files (x86)\\Battle.net"),
                PathBuf::from("C:\\Program Files\\Battle.net"),
            ];

            for path in standard_paths {
                if path.exists() {
                    log_debug!("Adding standard Battle.net path: {:?}", path);
                    paths_to_check.insert(0, path);
                }
            }
        }

        // 3. Base du launcher Battle.net
        let launcher_base = bnet_launcher_path.parent().unwrap_or(&bnet_launcher_path).to_path_buf();
        paths_to_check.push(launcher_base);

        log_debug!("📂 Found possible installation paths:");
        for path in &paths_to_check {
            log_debug!("   - {:?}", path);
        }

        for game_id in config.Games.keys() {
            if let Some(game_info) = self.known_games.get(game_id) {
                log_debug!("\n🎮 Checking game: {} ({})", game_info.name, game_id);

                let mut found = false;
                for base_path in &paths_to_check {
                    // Vérifier le chemin direct
                    let direct_path = base_path.join(game_info.game_dir);

                    // Vérifier dans les sous-dossiers (pour les tokens générés)
                    let mut possible_paths = vec![direct_path.clone()];
                    if let Ok(entries) = fs::read_dir(base_path) {
                        for entry in entries.flatten() {
                            if entry.path().is_dir() {
                                let game_path = entry.path().join(game_info.game_dir);
                                if game_path != direct_path {
                                    possible_paths.push(game_path);
                                }
                            }
                        }
                    }

                    for install_path in possible_paths {
                        let game_exe_path = install_path
                            .join(game_info.game_subdir)
                            .join(game_info.game_exe);

                        if game_exe_path.exists() {
                            log_debug!("    ✅ Found game at: {:?}", install_path);

                            if let Ok(size) = self.calculate_folder_size(&install_path) {
                                let launch_command = Self::create_launch_command(
                                    &bnet_launcher_path,
                                    &game_info.launch_code,
                                );

                                let game = Game {
                                    id: format!("battlenet_{}", game_id),
                                    platform_id: game_id.clone(),
                                    platform: Platform::BattleNet,
                                    title: game_info.name.to_string(),
                                    installation: GameInstallation {
                                        install_path: install_path.clone(),
                                        executable: Some(launch_command),
                                        size,
                                        version: None,
                                        last_updated: config.Games.get(game_id)
                                            .and_then(|g| g.LastActioned.as_ref())
                                            .and_then(|ts| ts.parse().ok()),
                                    },
                                    metadata: GameMetadata {
                                        title: game_info.name.to_string(),
                                        description: None,
                                        developer: Some("Blizzard Entertainment".to_string()),
                                        publisher: Some("Blizzard Entertainment".to_string()),
                                        release_date: None,
                                        genres: Vec::new(),
                                        tags: Vec::new(),
                                        media: None,
                                    },
                                    media: GameMedia {
                                        thumbnail: None,
                                        cover: None,
                                        screenshots: Vec::new(),
                                        background: None,
                                        icon: None,
                                        logo: None,
                                    },
                                    last_played: None,
                                    stats: GameStats {
                                        total_playtime: 0,
                                        last_session_duration: 0,
                                        sessions_count: 0,
                                        first_played: None,
                                        last_played: None,
                                    },
                                };

                                games.push(game);
                                found = true;
                                break;
                            }
                        }
                    }

                    if found {
                        break;
                    }
                }

                if !found {
                    log_debug!("    ❌ Game not found in any location: {}", game_info.name);
                }
            }
        }

        log_debug!("\n📊 Scan complete! Found {} games", games.len());
        Ok(games)
    }
}

use crate::models::{Game, GameInstallation, GameMedia, GameMetadata, GameResult, Platform, GameStats};
use crate::utils::AppError;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

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
        println!("🔍 Starting Battle.net games scan...");

        // Chemins standard pour le launcher
        let base_paths = vec![
            PathBuf::from("C:\\Program Files (x86)"),
            PathBuf::from("C:\\Program Files"),
        ];
        println!("📁 Checking base paths: {:?}", base_paths);

        // Trouver le launcher Battle.net
        let bnet_launcher_path = self.find_battlenet_launcher(&base_paths)?;
        println!("🎮 Found Battle.net launcher at: {:?}", bnet_launcher_path);

        // Obtenir le chemin des jeux depuis la config
        let config_path = Self::get_config_path()?;
        println!("📄 Reading config from: {:?}", config_path);

        let config_content = fs::read_to_string(&config_path).map_err(|e| AppError {
            message: format!("Erreur de lecture du fichier Battle.net.config: {}", e),
        })?;

        let config: BnetConfig = serde_json::from_str(&config_content).map_err(|e| AppError {
            message: format!("Erreur de parsing du fichier Battle.net.config: {}", e),
        })?;

        // Debug de la config
        println!("🔧 Found games in config:");
        for (game_id, game_config) in &config.Games {
            println!("    - Game ID: {}", game_id);
            println!("      ServerUid: {}", game_config.ServerUid);
            println!("      LastPlayed: {:?}", game_config.LastPlayed);
        }

        println!("📚 Known games in scanner:");
        for (id, info) in &self.known_games {
            println!("    - ID: {}", id);
            println!("      Name: {}", info.name);
            println!("      Directory: {}", info.game_dir);
        }

        // Trouver le chemin d'installation par défaut
        let default_path = config
            .Client
            .Install
            .as_ref()
            .map(|install| PathBuf::from(&install.DefaultInstallPath))
            .ok_or_else(|| AppError {
                message: "Chemin d'installation Battle.net non trouvé".to_string(),
            })?;

        println!("📂 Default installation path: {:?}", default_path);

        let mut games = Vec::new();

        // Parcourir les jeux connus
        for (game_id, game_info) in &self.known_games {
            println!("\n🎮 Checking game: {} ({})", game_info.name, game_id);

            // Vérifier si le jeu est installé selon la config Battle.net
            if let Some(bnet_game) = config.Games.get(game_id) {
                println!("    ✓ Found in Battle.net config");

                let install_path = default_path.join(game_info.game_dir);
                println!("    📁 Checking install path: {:?}", install_path);

                let game_exe_path = install_path
                    .join(game_info.game_subdir)
                    .join(game_info.game_exe);
                println!("    🎯 Looking for executable: {:?}", game_exe_path);

                if game_exe_path.exists() {
                    println!("    ✅ Executable found!");
                    match self.calculate_folder_size(&install_path) {
                        Ok(size) => {
                            println!("    💾 Size calculated: {} bytes", size);

                            let launch_command = Self::create_launch_command(
                                &bnet_launcher_path,
                                &game_info.launch_code,
                            );

                            let last_played = bnet_game
                                .LastPlayed
                                .as_ref()
                                .and_then(|ts| ts.parse::<i64>().ok());

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
                                    last_updated: bnet_game
                                        .LastActioned
                                        .as_ref()
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
                                last_played,
                                stats: GameStats {
                                    total_playtime: 0,
                                    last_session_duration: 0,
                                    sessions_count: 0,
                                    first_played: None,
                                    last_played: None,
                                },
                            };

                            println!("    ✨ Game added to list: {}", game_info.name);
                            games.push(game);
                        }
                        Err(e) => println!("    ❌ Error calculating size: {}", e),
                    }
                } else {
                    println!("    ❌ Executable not found!");
                }
            } else {
                println!("    ❌ Not found in Battle.net config!");
            }
        }

        println!("\n📊 Scan complete! Found {} games", games.len());
        Ok(games)
    }

//     pub async fn scan_games(&self) -> GameResult<Vec<Game>> {
//         // Chemins standard pour le launcher
//         let base_paths = vec![
//             PathBuf::from("C:\\Program Files (x86)"),
//             PathBuf::from("C:\\Program Files"),
//         ];
//
//         // Trouver le launcher Battle.net
//         let bnet_launcher_path = self.find_battlenet_launcher(&base_paths)?;
//
//         // Obtenir le chemin des jeux depuis la config
//         let config_path = Self::get_config_path()?;
//
//         let config_content = fs::read_to_string(&config_path).map_err(|e| AppError {
//             message: format!("Erreur de lecture du fichier Battle.net.config: {}", e),
//         })?;
//
//         let config: BnetConfig = serde_json::from_str(&config_content).map_err(|e| AppError {
//             message: format!("Erreur de parsing du fichier Battle.net.config: {}", e),
//         })?;
//
//         // Trouver le chemin d'installation par défaut
//         let default_path = config
//             .Client
//             .Install
//             .as_ref()
//             .map(|install| PathBuf::from(&install.DefaultInstallPath))
//             .ok_or_else(|| AppError {
//                 message: "Chemin d'installation Battle.net non trouvé".to_string(),
//             })?;
//
//         let mut games = Vec::new();
//
//         // Parcourir les jeux connus
//         for (game_id, game_info) in &self.known_games {
//             // Vérifier si le jeu est installé selon la config Battle.net
//             if let Some(bnet_game) = config.Games.get(game_id) {
//                 let install_path = default_path.join(game_info.game_dir);
//                 let game_exe_path = install_path
//                     .join(game_info.game_subdir)
//                     .join(game_info.game_exe);
//
//                 if game_exe_path.exists() {
//                     match self.calculate_folder_size(&install_path) {
//                         Ok(size) => {
//                             let launch_command = Self::create_launch_command(
//                                 &bnet_launcher_path,
//                                 &game_info.launch_code,
//                             );
//
//                             // Convertir le LastPlayed en timestamp si disponible
//                             let last_played = bnet_game
//                                 .LastPlayed
//                                 .as_ref()
//                                 .and_then(|ts| ts.parse::<i64>().ok());
//
//                             games.push(Game {
//                                 id: format!("battlenet_{}", game_id),
//                                 platform_id: game_id.clone(),
//                                 platform: Platform::BattleNet,
//                                 title: game_info.name.to_string(),
//                                 installation: GameInstallation {
//                                     install_path: install_path.clone(),
//                                     executable: Some(launch_command),
//                                     size,
//                                     version: None,
//                                     last_updated: bnet_game
//                                         .LastActioned
//                                         .as_ref()
//                                         .and_then(|ts| ts.parse().ok()),
//                                 },
//                                 metadata: GameMetadata {
//                                     title: game_info.name.to_string(),
//                                     description: None,
//                                     developer: Some("Blizzard Entertainment".to_string()),
//                                     publisher: Some("Blizzard Entertainment".to_string()),
//                                     release_date: None,
//                                     genres: Vec::new(),
//                                     tags: Vec::new(),
//                                     media: None,
//                                 },
//                                 media: GameMedia {
//                                     thumbnail: None,
//                                     cover: None,
//                                     screenshots: Vec::new(),
//                                     background: None,
//                                     icon: None,
//                                     logo: None,
//                                 },
//                                 last_played,
//                                 stats: GameStats {
//                                     total_playtime: 0,
//                                     last_session_duration: 0,
//                                     sessions_count: 0,
//                                     first_played: None,
//                                     last_played: None,
//                                 },
//                             });
//                         }
//                         Err(e) => println!("    ❌ Erreur lors du calcul de la taille: {}", e),
//                     }
//                 }
//             }
//         }
//         Ok(games)
//     }
}

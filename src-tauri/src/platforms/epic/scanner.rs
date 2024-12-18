// Dans platforms/epic/scanner.rs
use crate::models::{Game, GameInstallation, GameMedia, GameMetadata, GameResult, Platform, GameStats};
use crate::utils::AppError;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use winreg::enums::*;
use winreg::RegKey;

#[derive(Debug, Deserialize)]
struct EpicManifest {
    InstallLocation: String,
    DisplayName: String,
    AppName: String,
    #[serde(rename = "MainGameAppName")]
    main_executable: Option<String>,
    InstallSize: Option<u64>,
    #[serde(rename = "AppVersion")]
    version: Option<String>,
    #[serde(rename = "AppCategories")]
    categories: Option<Vec<String>>,
}

pub struct EpicGameScanner {
    install_path: PathBuf,
    user_added_folders: Mutex<Vec<PathBuf>>,
}

impl EpicGameScanner {
    pub fn new() -> Self {
        let install_path = Self::get_epic_path()
            .unwrap_or_else(|_| PathBuf::from("C:\\Program Files\\Epic Games"));

        Self {
            install_path,
            user_added_folders: Mutex::new(Vec::new()),
        }
    }

    pub fn get_install_path(&self) -> PathBuf {
        self.install_path.clone()
    }

    pub fn add_user_folder(&self, path: PathBuf) -> GameResult<()> {
        if let Ok(mut folders) = self.user_added_folders.lock() {
            if !folders.contains(&path) {
                folders.push(path);
            }
            Ok(())
        } else {
            Err(AppError {
                message: "Failed to lock user_added_folders".to_string(),
            })
        }
    }

    fn get_epic_path() -> GameResult<PathBuf> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let epic_key = hklm
            .open_subkey("SOFTWARE\\WOW6432Node\\Epic Games\\EpicGamesLauncher")
            .map_err(|e| AppError {
                message: format!("Failed to find Epic Games registry key: {}", e),
            })?;

        let install_path: String =
            epic_key
                .get_value("AppInstallLocation")
                .map_err(|e| AppError {
                    message: format!("Failed to get Epic Games install path: {}", e),
                })?;

        Ok(PathBuf::from(install_path))
    }

    fn get_manifests_path() -> GameResult<PathBuf> {
        let app_data = std::env::var("PROGRAMDATA").map_err(|_| AppError {
            message: "Failed to get PROGRAMDATA path".to_string(),
        })?;

        Ok(PathBuf::from(app_data)
            .join("Epic")
            .join("EpicGamesLauncher")
            .join("Data")
            .join("Manifests"))
    }

    pub async fn scan_games(&self) -> GameResult<Vec<Game>> {
        let manifests_path = Self::get_manifests_path()?;

        let mut games = Vec::new();

        if manifests_path.exists() {
            for entry in fs::read_dir(&manifests_path).map_err(|e| AppError {
                message: format!("Failed to read manifests directory: {}", e),
            })? {
                let entry = entry.map_err(|e| AppError {
                    message: format!("Failed to read directory entry: {}", e),
                })?;

                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("item") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        match serde_json::from_str::<EpicManifest>(&content) {
                            Ok(manifest) => {
                                if let Ok(game) = self.create_game_from_manifest(manifest) {
                                    games.push(game);
                                }
                            }
                            Err(e) => {
                                println!("❌ Failed to parse manifest {}: {}", path.display(), e);
                            }
                        }
                    }
                }
            }
        }
        Ok(games)
    }

    fn should_include_app(&self, manifest: &EpicManifest) -> bool {
        if let Some(categories) = &manifest.categories {
            categories
                .iter()
                .any(|cat| cat == "games" || cat == "applications")
        } else {
            false
        }
    }

    fn create_game_from_manifest(&self, manifest: EpicManifest) -> GameResult<Game> {
        if !self.should_include_app(&manifest) {
            return Err(AppError {
                message: format!("Skipping non-game application: {}", manifest.DisplayName),
            });
        }
        let install_path = PathBuf::from(&manifest.InstallLocation);

        let game = Game {
            id: format!("epic_{}", manifest.AppName),
            platform_id: manifest.AppName.clone(),
            platform: Platform::Epic,
            title: manifest.DisplayName.clone(),
            installation: GameInstallation {
                install_path: install_path.clone(),
                executable: Some(format!(
                    "com.epicgames.launcher://apps/{}?action=launch",
                    manifest.AppName
                )),
                size: manifest.InstallSize.unwrap_or(0),
                version: manifest.version,
                last_updated: None,
            },
            metadata: GameMetadata {
                title: manifest.DisplayName,
                description: None,
                developer: None,
                publisher: None,
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

        Ok(game)
    }
}

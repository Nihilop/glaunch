use crate::models::{Game, GameInstallation, GameMedia, GameMetadata, GameResult, Platform, GameStats};
use crate::utils::vdf::VdfParser;
use crate::utils::{acf::AcfParser, AppError};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::sync::Mutex;

pub struct SteamGameScanner {
    install_path: PathBuf,
    library_folders: Vec<PathBuf>,
    user_added_folders: Mutex<Vec<PathBuf>>,
}

impl SteamGameScanner {
    pub fn new() -> Self {
        let install_path = Self::get_steam_path()
            .unwrap_or_else(|_| PathBuf::from("C:\\Program Files (x86)\\Steam"));

        // Utiliser un HashSet pour éviter les doublons
        let mut library_folders = HashSet::new();

        // Ajouter le dossier Steam par défaut
        let default_library = install_path.join("steamapps");
        library_folders.insert(default_library);

        // Ajouter les bibliothèques supplémentaires depuis libraryfolders.vdf
        if let Ok(folders) = VdfParser::parse_library_folders(&install_path) {
            for folder in folders {
                if folder.mounted {
                    library_folders.insert(folder.path);
                }
            }
        }
        Self {
            install_path,
            library_folders: library_folders.into_iter().collect(),
            user_added_folders: Mutex::new(Vec::new()),
        }
    }

    fn get_steam_path() -> GameResult<PathBuf> {
        use winreg::enums::*;
        use winreg::RegKey;

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let steam_key = hklm
            .open_subkey("SOFTWARE\\WOW6432Node\\Valve\\Steam")
            .map_err(|e| AppError {
                message: format!("Failed to find Steam registry key: {}", e),
            })?;

        let install_path: String = steam_key.get_value("InstallPath").map_err(|e| AppError {
            message: format!("Failed to get Steam install path: {}", e),
        })?;

        Ok(PathBuf::from(install_path))
    }

    fn find_game_path(&self, install_dir: &str) -> GameResult<PathBuf> {
        // Vérifier d'abord dans le dossier "common" de chaque bibliothèque
        for library in &self.library_folders {
            let game_path = library.join("common").join(install_dir);
            if game_path.exists() {
                return Ok(game_path);
            }
        }

        // Vérifier aussi dans les dossiers utilisateur
        if let Ok(user_folders) = self.user_added_folders.try_lock() {
            for folder in user_folders.iter() {
                let game_path = folder.join("common").join(install_dir);
                if game_path.exists() {
                    return Ok(game_path);
                }
            }
        }

        Err(AppError {
            message: format!(
                "Could not find installation directory for game: {}",
                install_dir
            ),
        })
    }

    pub async fn parse_acf_file(&self, path: &Path) -> GameResult<Game> {
        let content = fs::read_to_string(path).map_err(|e| AppError {
            message: format!("Failed to read ACF file: {}", e),
        })?;

        let acf_parser = AcfParser::new(content);
        let acf_data = acf_parser.parse()?;

        // Essayer de trouver le chemin d'installation
        let game_path = self.find_game_path(&acf_data.install_dir)?;

        // On garde le jeu même si on ne trouve pas l'exécutable
        let executable = self
            .find_game_executable(&game_path, &acf_data.name)
            .ok()
            .map(String::from);

        Ok(Game {
            id: format!("steam_{}", acf_data.app_id),
            platform_id: acf_data.app_id,
            platform: Platform::Steam,
            title: acf_data.name.clone(),
            installation: GameInstallation {
                install_path: game_path,
                executable,
                size: acf_data.size_on_disk,
                version: Some(acf_data.buildid),
                last_updated: acf_data.last_updated,
            },
            metadata: GameMetadata {
                title: acf_data.name,
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
        })
    }

    pub fn get_library_paths(&self) -> Vec<PathBuf> {
        // Retourner toutes les bibliothèques, y compris les dossiers utilisateur
        let mut paths = self.library_folders.clone();
        if let Ok(user_folders) = self.user_added_folders.try_lock() {
            paths.extend(user_folders.iter().cloned());
        }
        paths
    }

    fn find_game_executable(&self, game_path: &Path, game_name: &str) -> GameResult<String> {
        use crate::utils::executable_finder::ExecutableFinder;

        let finder = ExecutableFinder::new();
        finder
            .find_main_executable(game_path, game_name)
            .map(|path| path.to_string_lossy().to_string())
    }

    pub async fn add_library_path(&self, path: PathBuf) -> GameResult<()> {
        if !path.exists() {
            return Err(AppError {
                message: format!("Path does not exist: {:?}", path),
            });
        }

        let mut folders = self.user_added_folders.lock().await;
        if !folders.contains(&path) {
            folders.push(path);
        }

        Ok(())
    }

    pub async fn scan_games(&self) -> GameResult<Vec<Game>> {
        let mut all_games = Vec::new();

        // Scanner toutes les bibliothèques Steam
        for library_path in &self.library_folders {
            if let Ok(entries) = fs::read_dir(library_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("acf") {
                        match self.parse_acf_file(&path).await {
                            Ok(game) => {
                                all_games.push(game);
                            }
                            Err(e) => eprintln!("Error parsing ACF file {:?}: {}", path, e),
                        }
                    }
                }
            }
        }

        // Scanner les dossiers ajoutés par l'utilisateur
        let user_folders = self.user_added_folders.lock().await;
        for folder in user_folders.iter() {
            if let Ok(entries) = fs::read_dir(folder) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("acf") {
                        if let Ok(game) = self.parse_acf_file(&path).await {
                            all_games.push(game);
                        }
                    }
                }
            }
        }

        Ok(all_games)
    }
}

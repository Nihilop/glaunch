use crate::utils::AppError;
use lazy_static::lazy_static;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct LibraryFolder {
    pub path: PathBuf,
    pub label: String,
    pub mounted: bool,
}

pub struct VdfParser;

impl VdfParser {
    pub fn parse_library_folders(steam_path: &Path) -> Result<Vec<LibraryFolder>, AppError> {
        let vdf_path = steam_path.join("steamapps/libraryfolders.vdf");
        let content = fs::read_to_string(&vdf_path).map_err(|e| AppError {
            message: format!("Failed to read libraryfolders.vdf: {}", e),
        })?;

        lazy_static! {
            // Mise à jour des regex pour mieux correspondre au format VDF
            static ref FOLDER_REGEX: Regex = Regex::new(r#""path"\s*"([^"]+)""#).unwrap();
            static ref LABEL_REGEX: Regex = Regex::new(r#""label"\s*"([^"]+)""#).unwrap();
        }

        let mut libraries = Vec::new();

        // Ajouter le dossier Steam par défaut
        libraries.push(LibraryFolder {
            path: steam_path.join("steamapps"),
            label: "Installation Steam".to_string(),
            mounted: true,
        });

        // Parser les bibliothèques supplémentaires
        for cap in FOLDER_REGEX.captures_iter(&content) {
            if let Some(path_match) = cap.get(1) {
                let path_str = path_match.as_str().replace("\\\\", "\\");
                let mut library_path = PathBuf::from(path_str.clone());
                library_path.push("steamapps");

                if library_path.exists() {
                    libraries.push(LibraryFolder {
                        path: library_path,
                        label: get_library_label(&path_str),
                        mounted: true,
                    });
                }
            }
        }

        Ok(libraries)
    }
}

fn get_library_label(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_string())
        .unwrap_or_else(|| "Bibliothèque Steam".to_string())
}

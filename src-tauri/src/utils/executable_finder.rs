use crate::utils::AppError;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct ExecutableFinder {
    // Liste des noms de fichiers à ignorer (launchers génériques, outils, etc.)
    ignored_executables: Vec<String>,
}

impl ExecutableFinder {
    pub fn new() -> Self {
        Self {
            ignored_executables: vec![
                "unins000.exe".to_string(),
                "UnityCrashHandler64.exe".to_string(),
                "UnrealCEFSubProcess.exe".to_string(),
                "launcher.exe".to_string(),
                "redist".to_string(),
                "dotnet".to_string(),
                "vcredist".to_string(),
                "directx".to_string(),
                "support".to_string(),
                "crash".to_string(),
                "_CommonRedist".to_string(),
            ],
        }
    }

    pub fn find_main_executable(
        &self,
        game_path: &Path,
        game_name: &str,
    ) -> Result<PathBuf, AppError> {
        let mut potential_exes = Vec::new();

        // Parcourir récursivement le dossier du jeu
        for entry in WalkDir::new(game_path)
            .max_depth(3) // Limiter la profondeur de recherche
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "exe" {
                    // Ignorer les exécutables de la liste noire
                    if !self.should_ignore_executable(path) {
                        potential_exes.push(path.to_path_buf());
                    }
                }
            }
        }

        // Si on n'a trouvé aucun exécutable
        if potential_exes.is_empty() {
            return Err(AppError {
                message: format!("No executable found for game: {}", game_name),
            });
        }

        // Trier les exécutables trouvés selon nos heuristiques
        self.rank_executables(&potential_exes, game_name)
    }

    fn should_ignore_executable(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Vérifier si le chemin contient un des motifs à ignorer
        self.ignored_executables
            .iter()
            .any(|ignored| path_str.contains(&ignored.to_lowercase()))
    }

    fn rank_executables(&self, exes: &[PathBuf], game_name: &str) -> Result<PathBuf, AppError> {
        // Créer un vecteur de tuples (executable, score)
        let mut scored_exes: Vec<(f32, &PathBuf)> = exes
            .iter()
            .map(|exe| (self.score_executable(exe, game_name), exe))
            .collect();

        // Trier par score décroissant
        scored_exes.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        // Prendre le meilleur score
        scored_exes
            .first()
            .map(|(_, exe)| (*exe).clone())
            .ok_or_else(|| AppError {
                message: format!("No suitable executable found for game: {}", game_name),
            })
    }

    fn score_executable(&self, exe: &Path, game_name: &str) -> f32 {
        let mut score = 0.0;
        let file_name = exe
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        let game_name = game_name.to_lowercase();

        // Le nom du fichier correspond exactement au nom du jeu
        if file_name == format!("{}.exe", game_name) {
            score += 10.0;
        }

        // Le nom du fichier contient le nom du jeu
        if file_name.contains(&game_name) {
            score += 5.0;
        }

        // Préférer les exécutables à la racine ou proche de la racine
        if let Ok(metadata) = fs::metadata(exe) {
            if metadata.len() > 1000000 {
                // Plus de 1MB
                score += 2.0;
            }
        }

        // Pénaliser les exécutables dans des sous-dossiers profonds
        if let Some(parent) = exe.parent() {
            let depth = parent.components().count();
            score -= depth as f32 * 0.5;
        }

        score
    }
}

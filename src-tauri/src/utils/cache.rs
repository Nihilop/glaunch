use crate::utils::AppError;
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tokio::fs as async_fs;
use tauri::path::BaseDirectory;
use crate::log_debug;
use tauri::AppHandle;
use tauri::Manager;

pub struct MediaCache {
    cache_dir: PathBuf,
    http_client: Client,
}

impl MediaCache {
    pub fn new(app: &AppHandle) -> Result<Self, AppError> {
        // Utiliser app_data_dir de tauri pour obtenir le bon chemin
        let cache_dir = app.path()
            .app_data_dir()
            .or_else(|_| Err(AppError {
                message: "Could not determine cache directory".to_string(),
            }))?
            .join("media");

        fs::create_dir_all(&cache_dir).map_err(|e| AppError {
            message: format!("Failed to create cache directory: {}", e),
        })?;

        Ok(Self {
            cache_dir,
            http_client: Client::new(),
        })
    }

   pub async fn get_or_download(&self, url: &str, max_age: Duration) -> Result<String, AppError> {
       // Générer la clé de cache
       let cache_key = format!("{:x}", {
           let mut hasher = Sha256::new();
           hasher.update(url.as_bytes());
           hasher.finalize()
       });

       // Construire les chemins
       let relative_path = format!("media/{}.jpg", cache_key); // Ajout de l'extension par défaut
       let cache_path = self.cache_dir.join(format!("{}.jpg", cache_key));

       log_debug!("Checking cache for: {}", &relative_path);

       // Vérifier le cache
       if let Ok(metadata) = fs::metadata(&cache_path) {
           if let Ok(modified) = metadata.modified() {
               if let Ok(age) = SystemTime::now().duration_since(modified) {
                   if age < max_age {
                       log_debug!("Cache hit: {}", &relative_path);
                       return Ok(relative_path);
                   }
               }
           }
       }

       // Si pas en cache ou expiré, télécharger
       log_debug!("Cache miss, downloading: {}", url);
       let response = self.http_client
           .get(url)
           .send()
           .await
           .map_err(|e| AppError {
               message: format!("Failed to download media: {}", e),
           })?;

       // Déterminer l'extension à partir du Content-Type
       let extension = response.headers()
           .get("content-type")
           .and_then(|ct| ct.to_str().ok())
           .map(|content_type| match content_type {
               "image/jpeg" | "image/jpg" => ".jpg",
               "image/png" => ".png",
               "image/gif" => ".gif",
               "image/webp" => ".webp",
               _ => ".jpg"
           })
           .unwrap_or(".jpg");

       // Mettre à jour les chemins avec la bonne extension
       let final_relative_path = format!("media/{}{}", cache_key, extension);
       let final_cache_path = self.cache_dir.join(format!("{}{}", cache_key, extension));

       // Télécharger et sauvegarder
       let bytes = response.bytes().await.map_err(|e| AppError {
           message: format!("Failed to read media bytes: {}", e),
       })?;

       async_fs::write(&final_cache_path, bytes)
           .await
           .map_err(|e| AppError {
               message: format!("Failed to write to cache: {}", e),
           })?;

       log_debug!("Saved new file: {} -> {}", url, &final_relative_path);

       Ok(final_relative_path)
   }

    pub fn clear_old_cache(&self, max_age: Duration) -> Result<(), AppError> {
        let now = SystemTime::now();

        let entries = fs::read_dir(&self.cache_dir).map_err(|e| AppError {
            message: format!("Failed to read cache directory: {}", e),
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| AppError {
                message: format!("Failed to read directory entry: {}", e),
            })?;

            let metadata = entry.metadata().map_err(|e| AppError {
                message: format!("Failed to read file metadata: {}", e),
            })?;

            if let Ok(modified) = metadata.modified() {
                if let Ok(age) = now.duration_since(modified) {
                    if age > max_age {
                        if let Err(e) = fs::remove_file(entry.path()) {
                            eprintln!("Failed to remove cached file: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

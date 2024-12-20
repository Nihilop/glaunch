use crate::utils::AppError;
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tokio::fs as async_fs;
use crate::log_debug;

pub struct MediaCache {
    cache_dir: PathBuf,
    http_client: Client,
}

impl MediaCache {
    pub fn new() -> Result<Self, AppError> {
        let cache_dir = Self::get_cache_dir()?;
        fs::create_dir_all(&cache_dir).map_err(|e| AppError {
            message: format!("Failed to create cache directory: {}", e),
        })?;

        Ok(Self {
            cache_dir,
            http_client: Client::new(),
        })
    }

    fn get_cache_dir() -> Result<PathBuf, AppError> {
        let base_dir = dirs::cache_dir().ok_or_else(|| AppError {
            message: "Could not determine cache directory".to_string(),
        })?;

        Ok(base_dir.join("glaunch").join("media"))
    }

    pub async fn get_or_download(&self, url: &str, max_age: Duration) -> Result<String, AppError> {
        let cache_key = format!("{:x}", {
            let mut hasher = Sha256::new();
            hasher.update(url.as_bytes());
            hasher.finalize()
        });

        // Vérifier d'abord le cache
        let cache_path = self.cache_dir.join(&cache_key);
        if let Ok(metadata) = fs::metadata(&cache_path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(age) = SystemTime::now().duration_since(modified) {
                    if age < max_age {
                        log_debug!("Using cached file: {}", cache_path.display());
                        return Ok(cache_path.to_string_lossy().to_string())
                    }
                }
            }
        }

        // Si pas en cache, télécharger
        let response = self.http_client
            .get(url)
            .send()
            .await
            .map_err(|e| AppError {
                message: format!("Failed to download media: {}", e),
            })?;

        let content_type = response.headers()
            .get("content-type")
            .and_then(|ct| ct.to_str().ok())
            .unwrap_or("image/jpeg");

        let extension = match content_type {
            "image/jpeg" | "image/jpg" => ".jpg",
            "image/png" => ".png",
            "image/gif" => ".gif",
            "image/webp" => ".webp",
            _ => ".jpg"  // extension par défaut
        };

        let cache_path = self.cache_dir.join(format!("{}{}", cache_key, extension));
        log_debug!("Cache path: {}", cache_path.display());

        if let Ok(metadata) = fs::metadata(&cache_path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(age) = SystemTime::now().duration_since(modified) {
                    if age < max_age {
                        return Ok(cache_path.to_string_lossy().to_string());
                    }
                }
            }
        }
        let response = self
            .http_client
            .get(url)
            .send()
            .await
            .map_err(|e| AppError {
                message: format!("Failed to download media: {}", e),
            })?;

        let bytes = response.bytes().await.map_err(|e| AppError {
            message: format!("Failed to read media bytes: {}", e),
        })?;

        async_fs::write(&cache_path, bytes)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to write to cache: {}", e),
            })?;

        let absolute_path = cache_path.to_string_lossy().replace('\\', "/");
        log_debug!("Absolute path for storage: {}", absolute_path);

        Ok(absolute_path)
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

// src/db/mod.rs
use crate::utils::AppError;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::PathBuf;
use std::time::Duration;
mod migrations;
mod queries;
mod schema;
use migrations::run_migrations;
use crate::log_debug;
use crate::log_error;

pub use queries::{GameQueries, MetadataQueries, SessionQueries};

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new() -> Result<Self, AppError> {
        let app_data = std::env::var("APPDATA").map_err(|_| AppError {
            message: "Failed to get APPDATA path".to_string(),
        })?;

        let db_dir = PathBuf::from(app_data).join("glaunch");
        std::fs::create_dir_all(&db_dir).map_err(|e| AppError {
            message: format!("Failed to create database directory: {}", e),
        })?;

        let db_path = db_dir.join("games.db");
        log_debug!("Database path: {}", db_path.display());

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(30))
            .idle_timeout(Duration::from_secs(60))
            .connect(&format!(
                "sqlite:{}?mode=rwc",
                db_path.to_str().ok_or_else(|| AppError {
                    message: "Invalid database path".to_string()
                })?
            ))
            .await
            .map_err(|e| AppError {
                message: format!("Failed to connect to database: {}", e),
            })?;

        // Désactiver le mode WAL
        sqlx::query("PRAGMA journal_mode = DELETE;")
            .execute(&pool)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to set journal mode: {}", e),
            })?;

        let db = Self { pool };

        // Initialiser la base de données
        log_debug!("Running database migrations...");
        match db.initialize().await {
            Ok(_) => {
                log_debug!("Database migrations completed successfully");
                Ok(db)
            },
            Err(e) => {
                log_error!("Database initialization failed: {}", e);
                Err(e)
            }
        }
    }

    pub async fn initialize(&self) -> Result<(), AppError> {
        log_debug!("Initializing database...");
        run_migrations(&self.pool).await?;
        log_debug!("Database initialized successfully");
        Ok(())
    }

    pub fn games(&self) -> GameQueries {
        GameQueries::new(&self.pool)
    }

    pub fn sessions(&self) -> SessionQueries {
        SessionQueries::new(&self.pool)
    }

    pub fn metadata(&self) -> MetadataQueries {
        MetadataQueries::new(&self.pool)
    }
}

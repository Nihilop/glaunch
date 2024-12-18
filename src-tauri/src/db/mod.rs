// src/db/mod.rs
use crate::utils::AppError;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::PathBuf;

mod migrations;
mod queries;
mod schema;
use migrations::run_migrations;

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

        // SQLite connection string for Windows
        let db_url = format!(
            "sqlite:{}?mode=rwc",
            db_path.to_str().ok_or_else(|| AppError {
                message: "Invalid database path".to_string()
            })?
        );

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to connect to database: {}", e),
            })?;

        run_migrations(&pool).await?;

        Ok(Self { pool })
    }

    pub async fn initialize(&self) -> Result<(), AppError> {
        // Exécuter les migrations
        run_migrations(&self.pool).await?;
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

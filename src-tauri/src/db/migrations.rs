// src/db/migrations.rs
use crate::utils::AppError;
use sqlx::Row;
use sqlx::SqlitePool;
use crate::log_debug;

pub struct Migration {
    version: i32,
    description: &'static str,
    up_sql: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        description: "Initial schema",
        up_sql: include_str!("./migrations/001_initial_schema.sql"),
    },
    // Les futures migrations seront ajoutées ici
];

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), AppError> {
    // Créer la table des migrations si elle n'existe pas
    log_debug!("Starting database migrations...");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            description TEXT NOT NULL,
            applied_at INTEGER NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| AppError {
        message: format!("Failed to create migrations table: {}", e),
    })?;

    log_debug!("Checking for pending migrations...");
    // Vérifier quelles migrations ont déjà été appliquées
    let applied_versions: Vec<i32> =
        sqlx::query("SELECT version FROM schema_migrations ORDER BY version")
            .fetch_all(pool)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch applied migrations: {}", e),
            })?
            .iter()
            .map(|row| row.get(0))
            .collect();

    // Appliquer les nouvelles migrations
    for migration in MIGRATIONS {
        if !applied_versions.contains(&migration.version) {
            let mut tx = pool.begin().await.map_err(|e| AppError {
                message: format!("Failed to start transaction: {}", e),
            })?;

            // Appliquer la migration
            sqlx::query(migration.up_sql)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError {
                    message: format!("Failed to apply migration {}: {}", migration.version, e),
                })?;

            // Enregistrer la migration comme appliquée
            sqlx::query(
                "INSERT INTO schema_migrations (version, description, applied_at) VALUES (?, ?, ?)",
            )
            .bind(migration.version)
            .bind(migration.description)
            .bind(chrono::Utc::now().timestamp())
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to record migration {}: {}", migration.version, e),
            })?;

            tx.commit().await.map_err(|e| AppError {
                message: format!("Failed to commit migration {}: {}", migration.version, e),
            })?;
        }
    }
    log_debug!("Migrations completed successfully");
    Ok(())
}

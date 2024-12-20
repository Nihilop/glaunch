// src/db/queries.rs
use crate::models::{Game, GameInstallation, GameMedia, GameMetadata, GameStats, Platform};
use crate::utils::AppError;
use chrono::Utc;
use sqlx::{Executor, Row, SqlitePool};
use std::path::PathBuf;

pub struct GameQueries<'a> {
    pool: &'a SqlitePool,
}

impl<'a> GameQueries<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_game(&self, game_id: &str) -> Result<Option<Game>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT
                g.*,
                m.title as meta_title,
                m.description,
                m.developer,
                m.publisher,
                m.release_date,
                med.thumbnail,
                med.cover,
                med.background,
                med.icon,
                med.logo,
                s.total_playtime,
                s.last_session_duration,
                s.sessions_count,
                s.first_played,
                s.last_played
            FROM games g
            LEFT JOIN game_metadata m ON g.id = m.game_id
            LEFT JOIN game_media med ON g.id = med.game_id
            LEFT JOIN game_stats s ON g.id = s.game_id
            WHERE g.id = ?
            "#,
        )
        .bind(game_id)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| AppError {
            message: format!("Failed to fetch game: {}", e),
        })?;

        match row {
            Some(row) => {
                // Récupérer les genres
                let genres = sqlx::query("SELECT genre FROM game_genres WHERE game_id = ?")
                    .bind(game_id)
                    .fetch_all(self.pool)
                    .await
                    .map_err(|e| AppError {
                        message: format!("Failed to fetch genres: {}", e),
                    })?
                    .iter()
                    .map(|r| r.get::<String, _>("genre"))
                    .collect();

                // Récupérer les tags
                let tags = sqlx::query("SELECT tag FROM game_tags WHERE game_id = ?")
                    .bind(game_id)
                    .fetch_all(self.pool)
                    .await
                    .map_err(|e| AppError {
                        message: format!("Failed to fetch tags: {}", e),
                    })?
                    .iter()
                    .map(|r| r.get::<String, _>("tag"))
                    .collect();

                // Récupérer les screenshots
                let screenshots = sqlx::query("SELECT url FROM game_screenshots WHERE game_id = ?")
                    .bind(game_id)
                    .fetch_all(self.pool)
                    .await
                    .map_err(|e| AppError {
                        message: format!("Failed to fetch screenshots: {}", e),
                    })?
                    .iter()
                    .map(|r| r.get::<String, _>("url"))
                    .collect();

                // Parser la platform
                let platform = match row.get::<String, _>("platform").as_str() {
                    "Steam" => Platform::Steam,
                    "BattleNet" => Platform::BattleNet,
                    "Epic" => Platform::Epic,
                    _ => Platform::Custom,
                };

                // Formater les chemins des médias
                let thumbnail = row.get::<Option<String>, _>("thumbnail");
                let cover = row.get::<Option<String>, _>("cover");
                let background = row.get::<Option<String>, _>("background");
                let icon = row.get::<Option<String>, _>("icon");
                let logo = row.get::<Option<String>, _>("logo");

                Ok(Some(Game {
                    id: row.get("id"),
                    platform_id: row.get("platform_id"),
                    platform,
                    title: row.get("title"),
                    installation: GameInstallation {
                        install_path: PathBuf::from(row.get::<String, _>("install_path")),
                        executable: row.get("executable"),
                        size: row.get::<i64, _>("install_size") as u64,
                        version: row.get("version"),
                        last_updated: row.get("last_updated"),
                    },
                    metadata: GameMetadata {
                        title: row
                            .get::<Option<String>, _>("meta_title")
                            .unwrap_or_else(|| row.get("title")),
                        description: row.get("description"),
                        developer: row.get("developer"),
                        publisher: row.get("publisher"),
                        release_date: row.get("release_date"),
                        genres,
                        tags,
                        media: None,
                    },
                    media: GameMedia {
                        thumbnail,
                        cover,
                        screenshots,
                        background,
                        icon,
                        logo,
                    },
                    last_played: row.get("last_played"),
                    stats: GameStats {
                        total_playtime: row.get::<Option<i64>, _>("total_playtime").unwrap_or(0),
                        last_session_duration: row
                            .get::<Option<i64>, _>("last_session_duration")
                            .unwrap_or(0),
                        sessions_count: row.get::<Option<i32>, _>("sessions_count").unwrap_or(0),
                        first_played: row.get("first_played"),
                        last_played: row.get("last_played"),
                    },
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn update_last_played(&self, game_id: &str, timestamp: i64) -> Result<(), AppError> {
        sqlx::query("UPDATE games SET last_played = ?, updated_at = ? WHERE id = ?")
            .bind(timestamp)
            .bind(timestamp)
            .bind(game_id)
            .execute(self.pool)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to update last_played: {}", e),
            })?;

        Ok(())
    }

    pub async fn delete_game(&self, game_id: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM games WHERE id = ?")
            .bind(game_id)
            .execute(self.pool)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to delete game: {}", e),
            })?;

        // Supprimer les données associées
        sqlx::query("DELETE FROM game_metadata WHERE game_id = ?")
            .bind(game_id)
            .execute(self.pool)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to delete game metadata: {}", e),
            })?;

        sqlx::query("DELETE FROM game_genres WHERE game_id = ?")
            .bind(game_id)
            .execute(self.pool)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to delete game genres: {}", e),
            })?;

        sqlx::query("DELETE FROM game_tags WHERE game_id = ?")
            .bind(game_id)
            .execute(self.pool)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to delete game tags: {}", e),
            })?;

        sqlx::query("DELETE FROM game_media WHERE game_id = ?")
            .bind(game_id)
            .execute(self.pool)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to delete game media: {}", e),
            })?;

        sqlx::query("DELETE FROM game_screenshots WHERE game_id = ?")
            .bind(game_id)
            .execute(self.pool)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to delete game screenshots: {}", e),
            })?;

        Ok(())
    }

    pub async fn get_all_games(&self) -> Result<Vec<Game>, AppError> {
        let game_ids: Vec<String> = sqlx::query("SELECT id FROM games ORDER BY title")
            .fetch_all(self.pool)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to fetch games: {}", e),
            })?
            .iter()
            .map(|row| row.get("id"))
            .collect();

        let mut games = Vec::new();
        for id in game_ids {
            if let Some(game) = self.get_game(&id).await? {
                games.push(game);
            }
        }

        Ok(games)
    }

    pub async fn upsert_game(&self, game: &Game) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(|e| AppError {
            message: format!("Failed to start transaction: {}", e),
        })?;

        let platform_str = match game.platform {
            Platform::Steam => "Steam",
            Platform::BattleNet => "BattleNet",
            Platform::Epic => "Epic",
            Platform::Custom => "Custom",
        };

        // Vérifier si le jeu existe
        let exists = sqlx::query("SELECT id FROM games WHERE id = ?")
            .bind(&game.id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to check game existence: {}", e),
            })?;

        // S'assurer que install_size est >= 0
        let install_size = if game.installation.size > 0 {
            game.installation.size as i64
        } else {
            0_i64
        };

        if exists.is_some() {
            // Update seulement les informations d'installation pour les jeux existants
            sqlx::query(
                r#"
                UPDATE games SET
                    install_path = ?,
                    executable = ?,
                    install_size = ?,
                    version = ?,
                    last_updated = ?,
                    updated_at = ?
                WHERE id = ?
                "#,
            )
            .bind(game.installation.install_path.to_string_lossy().to_string())
            .bind(&game.installation.executable)
            .bind(install_size)
            .bind(&game.installation.version)
            .bind(game.installation.last_updated)
            .bind(chrono::Utc::now().timestamp())
            .bind(&game.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to update game: {}", e),
            })?;
        } else {
            // Insert complet pour les nouveaux jeux
            sqlx::query(
                r#"
                INSERT INTO games (
                    id, platform_id, platform, title, install_path, executable,
                    install_size, version, last_updated, last_played, last_scan,
                    created_at, updated_at
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&game.id)
            .bind(&game.platform_id)
            .bind(platform_str)
            .bind(&game.title)
            .bind(game.installation.install_path.to_string_lossy().to_string())
            .bind(&game.installation.executable)
            .bind(install_size)
            .bind(&game.installation.version)
            .bind(game.installation.last_updated)
            .bind(game.last_played)
            .bind(chrono::Utc::now().timestamp())
            .bind(chrono::Utc::now().timestamp())
            .bind(chrono::Utc::now().timestamp())
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to insert game: {}", e),
            })?;
        }

        tx.commit().await.map_err(|e| AppError {
            message: format!("Failed to commit transaction: {}", e),
        })?;

        Ok(())
    }
}

pub struct SessionQueries<'a> {
    pool: &'a SqlitePool,
}

impl<'a> SessionQueries<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn start_session(&self, game_id: &str) -> Result<i64, AppError> {
        let now = Utc::now().timestamp();

        let row = sqlx::query(
            r#"
            INSERT INTO game_sessions (game_id, start_time)
            VALUES (?, ?)
            RETURNING id
            "#,
        )
        .bind(game_id)
        .bind(now)
        .fetch_one(self.pool)
        .await
        .map_err(|e| AppError {
            message: format!("Failed to start game session: {}", e),
        })?;

        Ok(row.get("id"))
    }

    pub async fn end_session(&self, session_id: i64, duration_secs: i64) -> Result<(), AppError> {
        let now = Utc::now().timestamp();

        let mut tx = self.pool.begin().await.map_err(|e| AppError {
            message: format!("Failed to start transaction: {}", e),
        })?;

        // Mettre à jour la session
        let row = sqlx::query(
            r#"
            UPDATE game_sessions
            SET end_time = ?, duration = ?
            WHERE id = ?
            RETURNING game_id, duration, start_time
            "#,
        )
        .bind(now)
        .bind(duration_secs)
        .bind(session_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError {
            message: format!("Failed to end game session: {}", e),
        })?;

        let game_id: String = row.get("game_id");
        let duration: i64 = row.get("duration");
        let start_time: i64 = row.get("start_time");

        // Vérifier les stats actuelles
        let current_stats =
            sqlx::query("SELECT total_playtime, sessions_count FROM game_stats WHERE game_id = ?")
                .bind(&game_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| AppError {
                    message: format!("Failed to fetch current stats: {}", e),
                })?;

        // Mettre à jour les statistiques globales
        sqlx::query(
            r#"
            INSERT INTO game_stats (
                game_id, total_playtime, last_session_duration,
                sessions_count, first_played, last_played
            )
            VALUES (?, ?, ?, 1, ?, ?)
            ON CONFLICT(game_id) DO UPDATE SET
                total_playtime = total_playtime + excluded.last_session_duration,
                last_session_duration = excluded.last_session_duration,
                sessions_count = sessions_count + 1,
                first_played = COALESCE(game_stats.first_played, excluded.first_played),
                last_played = excluded.last_played
            "#,
        )
        .bind(&game_id)
        .bind(duration)
        .bind(duration)
        .bind(start_time)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError {
            message: format!("Failed to update game stats: {}", e),
        })?;

        // Vérifier les nouvelles stats
        let new_stats =
            sqlx::query("SELECT total_playtime, sessions_count FROM game_stats WHERE game_id = ?")
                .bind(&game_id)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| AppError {
                    message: format!("Failed to fetch updated stats: {}", e),
                })?;

        tx.commit().await.map_err(|e| AppError {
            message: format!("Failed to commit transaction: {}", e),
        })?;

        Ok(())
    }

    pub async fn get_game_stats(&self, game_id: &str) -> Result<Option<GameStats>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT total_playtime, last_session_duration,
                   sessions_count, first_played, last_played
            FROM game_stats
            WHERE game_id = ?
            "#,
        )
        .bind(game_id)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| AppError {
            message: format!("Failed to fetch game stats: {}", e),
        })?;

        Ok(row.map(|r| GameStats {
            total_playtime: r.get("total_playtime"),
            last_session_duration: r.get("last_session_duration"),
            sessions_count: r.get("sessions_count"),
            first_played: r.get("first_played"),
            last_played: r.get("last_played"),
        }))
    }
}

pub struct MetadataQueries<'a> {
    pool: &'a SqlitePool,
}

impl<'a> MetadataQueries<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_metadata(&self, game_id: &str) -> Result<Option<GameMetadata>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT
                m.title, m.description, m.developer, m.publisher,
                m.release_date,
                med.thumbnail, med.cover, med.background, med.icon, med.logo
            FROM game_metadata m
            LEFT JOIN game_media med ON m.game_id = med.game_id
            WHERE m.game_id = ?
            "#,
        )
        .bind(game_id)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| AppError {
            message: format!("Failed to fetch metadata: {}", e),
        })?;

        if let Some(row) = row {
            // Récupérer les genres
            let genres = sqlx::query("SELECT genre FROM game_genres WHERE game_id = ?")
                .bind(game_id)
                .fetch_all(self.pool)
                .await
                .map_err(|e| AppError {
                    message: format!("Failed to fetch genres: {}", e),
                })?
                .iter()
                .map(|r| r.get::<String, _>("genre"))
                .collect();

            // Récupérer les tags
            let tags = sqlx::query("SELECT tag FROM game_tags WHERE game_id = ?")
                .bind(game_id)
                .fetch_all(self.pool)
                .await
                .map_err(|e| AppError {
                    message: format!("Failed to fetch tags: {}", e),
                })?
                .iter()
                .map(|r| r.get::<String, _>("tag"))
                .collect();

            let media = GameMedia {
                thumbnail: row.get("thumbnail"),
                cover: row.get("cover"),
                background: row.get("background"),
                icon: row.get("icon"),
                logo: row.get("logo"),
                screenshots: Vec::new(), // On pourrait les ajouter si nécessaire
            };

            Ok(Some(GameMetadata {
                title: row.get("title"),
                description: row.get("description"),
                developer: row.get("developer"),
                publisher: row.get("publisher"),
                release_date: row.get("release_date"),
                genres,
                tags,
                media: Some(media),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn update_metadata(
        &self,
        game_id: &str,
        metadata: &GameMetadata,
        media: Option<&GameMedia>,
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(|e| AppError {
            message: format!("Failed to start transaction: {}", e),
        })?;

        // Mise à jour des métadonnées principales
        sqlx::query(
            r#"
                INSERT INTO game_metadata (
                    game_id, title, description, developer,
                    publisher, release_date, last_fetched
                )
                VALUES (?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(game_id) DO UPDATE SET
                    title = excluded.title,
                    description = excluded.description,
                    developer = excluded.developer,
                    publisher = excluded.publisher,
                    release_date = excluded.release_date,
                    last_fetched = excluded.last_fetched
                "#,
        )
        .bind(game_id)
        .bind(&metadata.title)
        .bind(&metadata.description)
        .bind(&metadata.developer)
        .bind(&metadata.publisher)
        .bind(&metadata.release_date)
        .bind(chrono::Utc::now().timestamp())
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError {
            message: format!("Failed to update metadata: {}", e),
        })?;

        // Mettre à jour les genres - d'abord supprimer les existants
        sqlx::query("DELETE FROM game_genres WHERE game_id = ?")
            .bind(game_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to clear genres: {}", e),
            })?;

        // Insérer les nouveaux genres avec gestion des doublons
        let mut unique_genres = std::collections::HashSet::new();
        for genre in &metadata.genres {
            if unique_genres.insert(genre) {
                // Uniquement insérer si pas déjà présent
                sqlx::query("INSERT INTO game_genres (game_id, genre) VALUES (?, ?)")
                    .bind(game_id)
                    .bind(genre)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| AppError {
                        message: format!("Failed to insert genre: {}", e),
                    })?;
            }
        }

        // Mettre à jour les tags - d'abord supprimer les existants
        sqlx::query("DELETE FROM game_tags WHERE game_id = ?")
            .bind(game_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to clear tags: {}", e),
            })?;

        // Insérer les nouveaux tags avec gestion des doublons
        let mut unique_tags = std::collections::HashSet::new();
        for tag in &metadata.tags {
            if unique_tags.insert(tag) {
                // Uniquement insérer si pas déjà présent
                sqlx::query("INSERT INTO game_tags (game_id, tag) VALUES (?, ?)")
                    .bind(game_id)
                    .bind(tag)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| AppError {
                        message: format!("Failed to insert tag: {}", e),
                    })?;
            }
        }

        // Mise à jour de la table game_media
        if let Some(media) = media {
            sqlx::query(
                r#"
                    INSERT INTO game_media (
                        game_id, thumbnail, cover, background, icon, logo, last_fetched
                    )
                    VALUES (?, ?, ?, ?, ?, ?, ?)
                    ON CONFLICT(game_id) DO UPDATE SET
                        thumbnail = excluded.thumbnail,
                        cover = excluded.cover,
                        background = excluded.background,
                        icon = excluded.icon,
                        logo = excluded.logo,
                        last_fetched = excluded.last_fetched
                    "#,
            )
            .bind(game_id)
            .bind(&media.thumbnail)
            .bind(&media.cover)
            .bind(&media.background)
            .bind(&media.icon)
            .bind(&media.logo)
            .bind(chrono::Utc::now().timestamp())
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError {
                message: format!("Failed to update media: {}", e),
            })?;

            // Mise à jour des screenshots
            sqlx::query("DELETE FROM game_screenshots WHERE game_id = ?")
                .bind(game_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError {
                    message: format!("Failed to clear screenshots: {}", e),
                })?;

            for screenshot in &media.screenshots {
                sqlx::query("INSERT INTO game_screenshots (game_id, url) VALUES (?, ?)")
                    .bind(game_id)
                    .bind(screenshot)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| AppError {
                        message: format!("Failed to insert screenshot: {}", e),
                    })?;
            }
        }

        tx.commit().await.map_err(|e| AppError {
            message: format!("Failed to commit transaction: {}", e),
        })?;

        Ok(())
    }
}

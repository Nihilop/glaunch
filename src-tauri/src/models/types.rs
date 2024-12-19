// Dans src/models/types.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Platform {
    Steam,
    BattleNet,
    Epic,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMedia {
    pub thumbnail: Option<String>,
    pub cover: Option<String>,
    pub screenshots: Vec<String>,
    pub background: Option<String>,
    pub icon: Option<String>,
    pub logo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMetadata {
    pub title: String,
    pub description: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub release_date: Option<String>,
    pub genres: Vec<String>,
    pub tags: Vec<String>,
    pub media: Option<GameMedia>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInstallation {
    #[serde(rename = "path")]
    pub install_path: PathBuf,
    pub executable: Option<String>,
    #[serde(rename = "install_size")]
    pub size: u64,
    pub version: Option<String>,
    pub last_updated: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStats {
    pub total_playtime: i64,
    pub last_session_duration: i64,
    pub sessions_count: i32,
    pub first_played: Option<i64>,
    pub last_played: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    pub platform_id: String,
    pub platform: Platform,
    #[serde(rename = "name")]
    pub title: String, // Ce champ sera sérialisé comme "name" pour le frontend
    pub installation: GameInstallation,
    pub metadata: GameMetadata,
    pub media: GameMedia,
    pub last_played: Option<i64>,
    pub stats: GameStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomGameConfig {
    pub title: String,
    pub executable_path: String,
    pub install_path: String,
    pub platform: String,
    pub icon_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStatus {
    pub current_version: String,
    pub latest_version: Option<String>,
    pub update_available: bool,
    pub last_check: Option<i64>,
    pub download_url: Option<String>,
    pub release_notes: Option<String>,
    pub checking: bool,
}

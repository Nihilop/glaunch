use crate::utils::AppError;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct AcfParser {
    content: String,
}

#[derive(Debug)]
pub struct AcfData {
    pub app_id: String,
    pub name: String,
    pub install_dir: String,
    pub size_on_disk: u64,
    pub buildid: String,
    pub last_updated: Option<i64>,
    pub state: u32,
}

#[derive(Debug)]
pub struct LibraryFolder {
    pub path: PathBuf,
    pub label: String,
    pub mounted: bool,
    pub size: u64,
}

impl AcfParser {
    pub fn new(content: String) -> Self {
        Self { content }
    }

    pub fn parse(&self) -> Result<AcfData, AppError> {
        lazy_static! {
            static ref KEY_VALUE: Regex = Regex::new(r#""\s*([^"]+)\s*"\s*"([^"]*)""#).unwrap();
        }

        let mut data = HashMap::new();

        // Parse chaque ligne du fichier ACF
        for line in self.content.lines() {
            if let Some(caps) = KEY_VALUE.captures(line) {
                let key = caps.get(1).unwrap().as_str().trim();
                let value = caps.get(2).unwrap().as_str().trim();
                data.insert(key.to_string(), value.to_string());
            }
        }

        // Extraire les données nécessaires
        Ok(AcfData {
            app_id: data
                .get("appid")
                .ok_or_else(|| AppError {
                    message: "Missing appid".to_string(),
                })?
                .clone(),

            name: data
                .get("name")
                .ok_or_else(|| AppError {
                    message: "Missing name".to_string(),
                })?
                .clone(),

            install_dir: data
                .get("installdir")
                .ok_or_else(|| AppError {
                    message: "Missing installdir".to_string(),
                })?
                .clone(),

            size_on_disk: data
                .get("SizeOnDisk")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),

            buildid: data.get("buildid").unwrap_or(&"0".to_string()).clone(),

            last_updated: data.get("LastUpdated").and_then(|s| s.parse().ok()),

            state: data
                .get("StateFlags")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
        })
    }
}

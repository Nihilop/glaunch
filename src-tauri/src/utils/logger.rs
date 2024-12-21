use crate::utils::AppError;
use chrono::Local;
use lazy_static::lazy_static;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::AppHandle;
use crate::utils::AppPaths;

lazy_static! {
    static ref LOG_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);
}

#[derive(Debug)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Debug,
}

pub struct Logger;

impl Logger {
    pub fn init(app_handle: &AppHandle) -> Result<(), AppError> {
        let paths = AppPaths::new(app_handle)?;
        let log_path = paths.get_log_path();

        // S'assurer que le répertoire des logs existe
        if let Some(parent) = log_path.parent() {
            create_dir_all(parent).map_err(|e| AppError {
                message: format!("Failed to create log directory: {}", e),
            })?;
        }

        // Stocker le chemin avant d'ouvrir le fichier
        {
            let mut path_guard = LOG_PATH.lock().unwrap();
            *path_guard = Some(log_path.clone());
        }

        // Ouvrir et initialiser le fichier de log
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(|e| AppError {
                message: format!("Failed to open log file: {}", e),
            })?;

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        writeln!(
            file,
            "\n[{}] [INFO] Application starting... Version: {}",
            timestamp,
            env!("CARGO_PKG_VERSION")
        ).map_err(|e| AppError {
            message: format!("Failed to write to log file: {}", e),
        })?;

        // Écrire un log de test pour confirmer l'initialisation
        Self::log(
            LogLevel::Info,
            &format!("Logger initialized successfully at: {}", log_path.display())
        );

        Ok(())
    }

    pub fn log(level: LogLevel, message: &str) {
        if let Ok(guard) = LOG_PATH.lock() {
            if let Some(log_path) = guard.as_ref() {
                if let Ok(mut file) = OpenOptions::new().append(true).open(log_path) {
                    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
                    let level_str = match level {
                        LogLevel::Info => "INFO",
                        LogLevel::Warning => "WARN",
                        LogLevel::Error => "ERROR",
                        LogLevel::Debug => "DEBUG",
                    };
                    if let Err(e) = writeln!(file, "[{}] [{}] {}", timestamp, level_str, message) {
                        eprintln!("Failed to write to log file: {}", e);
                    }
                }
            }
        }
    }
}

// Macros pour faciliter le logging
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        crate::utils::logger::Logger::log(
            crate::utils::logger::LogLevel::Info,
            &format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        crate::utils::logger::Logger::log(
            crate::utils::logger::LogLevel::Warning,
            &format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        crate::utils::logger::Logger::log(
            crate::utils::logger::LogLevel::Error,
            &format!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        crate::utils::logger::Logger::log(
            crate::utils::logger::LogLevel::Debug,
            &format!($($arg)*)
        )
    };
}

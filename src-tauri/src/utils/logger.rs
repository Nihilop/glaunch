use crate::utils::AppError;
use chrono::Local;
use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref LOG_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);
}

#[derive(Debug)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Debug
}

pub struct Logger;

impl Logger {
    pub fn init() -> Result<(), AppError> {
        let app_data = std::env::var("APPDATA").map_err(|e| AppError {
            message: format!("Failed to get APPDATA path: {}", e),
        })?;

        let log_dir = PathBuf::from(app_data).join("glaunch").join("logs");
        create_dir_all(&log_dir).map_err(|e| AppError {
            message: format!("Failed to create log directory: {}", e),
        })?;

        let log_path = log_dir.join("app.log");

        // Initialize log file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(|e| AppError {
                message: format!("Failed to open log file: {}", e),
            })?;

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        writeln!(file, "\n[{}] [INFO] Application starting... Version: {}", timestamp, env!("CARGO_PKG_VERSION"))
            .map_err(|e| AppError {
                message: format!("Failed to write to log file: {}", e),
            })?;

        // Store the log path
        let mut path = LOG_PATH.lock().unwrap();
        *path = Some(log_path);

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
                    let _ = writeln!(file, "[{}] [{}] {}", timestamp, level_str, message);
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
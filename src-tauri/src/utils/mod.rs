use std::error::Error;
use std::fmt;
pub mod acf;
pub mod cache;
pub mod executable_finder;
pub mod logger;
pub mod settings;
pub mod vdf;
pub mod secrets;

pub use logger::Logger;

#[derive(Debug)]
pub struct AppError {
    pub message: String,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AppError {}

// Garder une seule implémentation From<String>
impl From<String> for AppError {
    fn from(message: String) -> Self {
        Self { message }
    }
}

impl From<&str> for AppError {
    fn from(message: &str) -> Self {
        Self { message: message.to_string() }
    }
}

pub fn to_app_error<E: std::error::Error>(err: E) -> AppError {
    AppError {
        message: err.to_string(),
    }
}

impl From<reqwest::Error> for AppError {
    fn from(error: reqwest::Error) -> Self {
        AppError {
            message: format!("Request failed: {}", error),
        }
    }
}
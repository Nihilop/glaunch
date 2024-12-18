pub mod types;
pub use types::*;

use crate::utils::AppError;

// Type de résultat générique pour notre application
pub type GameResult<T> = Result<T, AppError>;

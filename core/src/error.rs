//! Error types for the kabtangan-core engine.

use thiserror::Error;

/// Top-level error type for the core engine.
#[derive(Debug, Error)]
pub enum CoreError {
    /// Storage / SQLite errors.
    #[error("storage error: {0}")]
    Storage(#[from] rusqlite::Error),

    /// Entry not found in the dictionary.
    #[error("word not found: {0}")]
    NotFound(String),

    /// Invalid input passed to a function.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// I/O errors (e.g., reading external dictionary files).
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization errors.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Convenience alias.
pub type CoreResult<T> = Result<T, CoreError>;

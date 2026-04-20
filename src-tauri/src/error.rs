use serde::{Serialize, Serializer};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serde JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("API request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Environment variable error: {0}")]
    Env(String),

    #[error("API error {status}: {body}")]
    Api { status: u16, body: String },

    #[error("AI error: {0}")]
    Ai(String),

    #[error("Internal error: {0}")]
    #[allow(dead_code)]
    Internal(String),

    #[error("Cancelled: {0}")]
    Cancelled(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;

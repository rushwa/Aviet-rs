use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Server error: {0}")]
    Server(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid response")]
    InvalidResponse,

    #[error("Session not found")]
    SessionNotFound,

    #[error("Browser error: {0}")]
    Browser(String),
}

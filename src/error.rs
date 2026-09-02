use thiserror::Error;
use crate::enums::{Language, Theme};

/// Result alias for Akinator operations.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Errors that can occur during Akinator interactions.
#[derive(Debug, Error)]
pub enum Error {
    #[error("HTTP client error: {0}")]
    Http(#[from] wreq::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Regex compilation/matching error: {0}")]
    Regex(#[from] regex::Error),

    #[error("URL parse error: {0}")]
    Url(#[from] url::ParseError),

    #[error("Game session has not been started yet. Call start() first.")]
    SessionNotStarted,

    #[error("A guess has already been made. Call continue_game() or submit_win().")]
    SessionAlreadyWon,

    #[error("Akinator has given up and has no more questions.")]
    SessionGivenUp,

    #[error("Cannot go back any further. Already on the first question.")]
    CantGoBack,

    #[error("Session has expired or returned an invalid state.")]
    SessionExpired,

    #[error("Invalid answer input: '{0}'. Expected values like 'yes', 'no', 'idk', 'probably', 'probably not', or 0..=4.")]
    InvalidAnswer(String),

    #[error("Invalid language input: '{0}'.")]
    InvalidLanguage(String),

    #[error("Invalid theme input: '{0}'.")]
    InvalidTheme(String),

    #[error("Theme '{theme}' is not supported for language '{language}'.")]
    ThemeNotSupportedForLanguage {
        theme: Theme,
        language: Language,
    },

    #[error("Akinator served a Cloudflare anti-bot challenge on '{endpoint}'.")]
    CloudflareBlocked {
        endpoint: String,
    },

    #[error("Failed to extract required session parameter: {0}")]
    ExtractionFailed(String),

    #[error("Akinator API error: {0}")]
    AkinatorError(String),
}

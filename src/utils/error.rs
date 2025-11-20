use colored::Colorize;
use std::fmt;

/// Custom error type for rshare
#[derive(Debug)]
pub enum Error {
    FileError(String),    // File I/O and permissions
    NetworkError(String), // Network and connections issues
    CryptoError(String),  // Cryptography and signing failures
    InvalidInput(String), // Invalid user input or parameters
    ConfigError(String),  // Configuration loading and parsing errors
    SessionError(String), // Session setup and transfer issues
    UnknownIssue(String), // Catch-all for unexpected errors
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::FileError(msg) => write!(f, "File Error: {}", msg.red().underline()),
            Error::NetworkError(msg) => {
                write!(f, "Network Error: {}", msg.red().underline())
            }
            Error::CryptoError(msg) => write!(f, "Cryptography Error: {}", msg.red().underline()),
            Error::InvalidInput(msg) => {
                write!(f, "Invalid Input: {}", msg.red().underline())
            }
            Error::ConfigError(msg) => write!(f, "Configuration Error: {}", msg.red().underline()),
            Error::SessionError(msg) => {
                write!(f, "Session Error: {}", msg.red().underline())
            }
            Error::UnknownIssue(msg) => {
                write!(f, "Unknown Issue: {}", msg.red().underline())
            }
        }
    }
}

impl std::error::Error for Error {}

/// Convert std::io::Error to our Error type
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => Error::FileError(err.to_string()),
            std::io::ErrorKind::PermissionDenied => Error::FileError(err.to_string()),
            _ => Error::NetworkError(err.to_string()),
        }
    }
}

/// Convert toml errors
impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::ConfigError(err.to_string())
    }
}

impl From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Self {
        Error::ConfigError(err.to_string())
    }
}

/// Convert serde_json errors
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::ConfigError(err.to_string())
    }
}

/// Convert reqwest errors
impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::NetworkError(err.to_string())
    }
}

/// Custom Result type
pub type Result<T> = std::result::Result<T, Error>;

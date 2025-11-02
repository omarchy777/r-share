use std::fmt;

/// Custom error type for rshare
#[derive(Debug)]
pub enum Error {
    /// File system errors
    FileNotFound(String),
    FileRead(String),
    FileWrite(String),

    /// Network errors
    NetworkError(String),
    ConnectionFailed(String),

    /// Crypto errors
    KeyGenerationFailed(String),

    /// Input validation errors
    InvalidInput(String),

    /// Configuration errors
    ConfigError(String),
    CryptoError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::FileNotFound(msg) => write!(f, "File not found: {}", msg),
            Error::FileRead(msg) => write!(f, "Failed to read file: {}", msg),
            Error::FileWrite(msg) => write!(f, "Failed to write file: {}", msg),
            Error::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Error::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            Error::KeyGenerationFailed(msg) => write!(f, "Key generation failed: {}", msg),
            Error::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            Error::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
            Error::CryptoError(msg) => write!(f, "Crypto error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

/// Convert std::io::Error to our Error type
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => Error::FileNotFound(err.to_string()),
            std::io::ErrorKind::PermissionDenied => Error::FileRead(err.to_string()),
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

/// Custom Result type
pub type Result<T> = std::result::Result<T, Error>;

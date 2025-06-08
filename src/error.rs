use std::fmt;

#[derive(Debug)]
#[allow(dead_code)]
pub enum PassmanError {
    // Crypto errors
    Crypto(crate::crypto::CryptoError),
    
    // IO errors
    Io(std::io::Error),
    
    // Serialization errors
    Serialization(serde_json::Error),
    
    // Vault errors
    VaultNotFound(String),
    VaultAlreadyExists(String),
    EntryNotFound(String),
    EntryAlreadyExists(String),
    
    // Authentication errors
    InvalidPassword,
    WeakPassword(String),
    
    // Validation errors
    InvalidInput(String),
    
    // Clipboard errors
    Clipboard(String),
}

impl fmt::Display for PassmanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PassmanError::Crypto(e) => write!(f, "Cryptographic error: {}", e),
            PassmanError::Io(e) => write!(f, "IO error: {}", e),
            PassmanError::Serialization(e) => write!(f, "Serialization error: {}", e),
            PassmanError::VaultNotFound(path) => write!(f, "Vault not found: {}", path),
            PassmanError::VaultAlreadyExists(path) => write!(f, "Vault already exists: {}", path),
            PassmanError::EntryNotFound(id) => write!(f, "Entry not found: {}", id),
            PassmanError::EntryAlreadyExists(id) => write!(f, "Entry already exists: {}", id),
            PassmanError::InvalidPassword => write!(f, "Invalid password"),
            PassmanError::WeakPassword(msg) => write!(f, "Weak password: {}", msg),
            PassmanError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            PassmanError::Clipboard(msg) => write!(f, "Clipboard error: {}", msg),
        }
    }
}

impl std::error::Error for PassmanError {}

impl From<crate::crypto::CryptoError> for PassmanError {
    fn from(err: crate::crypto::CryptoError) -> Self {
        PassmanError::Crypto(err)
    }
}

impl From<std::io::Error> for PassmanError {
    fn from(err: std::io::Error) -> Self {
        PassmanError::Io(err)
    }
}

impl From<serde_json::Error> for PassmanError {
    fn from(err: serde_json::Error) -> Self {
        PassmanError::Serialization(err)
    }
}

pub type Result<T> = std::result::Result<T, PassmanError>;

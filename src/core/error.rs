//! Unified Error Handling Module
//!
//! Provides a typed error hierarchy for the password manager application
//! with user-friendly display messages and automatic conversion from
//! underlying library errors.

use std::fmt;
use std::io;

/// Main error type for the password manager
#[derive(Debug)]
pub enum PassmanError {
    /// Vault-related errors
    Vault(VaultError),
    /// Cryptographic errors
    Crypto(CryptoError),
    /// Import/Export errors
    Transfer(TransferError),
    /// IO errors
    Io(io::Error),
    /// Other errors
    Other(String),
}

/// Vault operation errors
#[derive(Debug, Clone)]
pub enum VaultError {
    /// Vault file not found
    NotFound(String),
    /// Vault already exists
    AlreadyExists(String),
    /// Vault is corrupted
    Corrupted(String),
    /// Vault integrity check failed
    IntegrityFailed,
    /// Failed to read vault
    ReadError(String),
    /// Failed to write vault
    WriteError(String),
    /// Entry not found
    EntryNotFound(String),
    /// Entry already exists
    EntryExists(String),
    /// Invalid vault format
    InvalidFormat(String),
}

/// Cryptographic errors
#[derive(Debug, Clone)]
pub enum CryptoError {
    /// Key derivation failed
    KeyDerivation(String),
    /// Encryption failed
    Encryption(String),
    /// Decryption failed (usually wrong password)
    Decryption(String),
    /// Invalid salt
    InvalidSalt(String),
    /// HMAC verification failed
    HmacVerification,
}

/// Import/Export errors
#[derive(Debug, Clone)]
pub enum TransferError {
    /// Unsupported format
    UnsupportedFormat(String),
    /// Parse error
    ParseError(String),
    /// File not found
    FileNotFound(String),
    /// Permission denied
    PermissionDenied(String),
    /// Invalid data
    InvalidData(String),
}

impl fmt::Display for PassmanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PassmanError::Vault(e) => write!(f, "{}", e),
            PassmanError::Crypto(e) => write!(f, "{}", e),
            PassmanError::Transfer(e) => write!(f, "{}", e),
            PassmanError::Io(e) => write!(f, "IO error: {}", e),
            PassmanError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl fmt::Display for VaultError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VaultError::NotFound(path) => write!(f, "Vault not found: '{}'. Run 'passman init' to create one.", path),
            VaultError::AlreadyExists(path) => write!(f, "Vault '{}' already exists. Remove it first or choose a different name.", path),
            VaultError::Corrupted(msg) => write!(f, "Vault file is corrupted: {}", msg),
            VaultError::IntegrityFailed => write!(f, "Vault integrity check failed. The file may have been tampered with."),
            VaultError::ReadError(msg) => write!(f, "Failed to read vault: {}", msg),
            VaultError::WriteError(msg) => write!(f, "Failed to write vault: {}", msg),
            VaultError::EntryNotFound(id) => write!(f, "Entry '{}' not found.", id),
            VaultError::EntryExists(id) => write!(f, "Entry '{}' already exists. Use 'edit' to modify it.", id),
            VaultError::InvalidFormat(msg) => write!(f, "Invalid vault format: {}", msg),
        }
    }
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CryptoError::KeyDerivation(msg) => write!(f, "Key derivation failed: {}", msg),
            CryptoError::Encryption(msg) => write!(f, "Encryption failed: {}", msg),
            CryptoError::Decryption(_) => write!(f, "Decryption failed. Please check your master password."),
            CryptoError::InvalidSalt(msg) => write!(f, "Invalid salt: {}", msg),
            CryptoError::HmacVerification => write!(f, "Vault authentication failed. The file may have been tampered with."),
        }
    }
}

impl fmt::Display for TransferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransferError::UnsupportedFormat(fmt) => write!(f, "Unsupported format: '{}'. Use json, csv, or chrome.", fmt),
            TransferError::ParseError(msg) => write!(f, "Failed to parse file: {}", msg),
            TransferError::FileNotFound(path) => write!(f, "File not found: '{}'", path),
            TransferError::PermissionDenied(path) => write!(f, "Permission denied: '{}'", path),
            TransferError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
        }
    }
}

impl std::error::Error for PassmanError {}

// Conversion implementations
impl From<io::Error> for PassmanError {
    fn from(err: io::Error) -> Self {
        PassmanError::Io(err)
    }
}

impl From<VaultError> for PassmanError {
    fn from(err: VaultError) -> Self {
        PassmanError::Vault(err)
    }
}

impl From<CryptoError> for PassmanError {
    fn from(err: CryptoError) -> Self {
        PassmanError::Crypto(err)
    }
}

impl From<TransferError> for PassmanError {
    fn from(err: TransferError) -> Self {
        PassmanError::Transfer(err)
    }
}

impl From<String> for PassmanError {
    fn from(msg: String) -> Self {
        PassmanError::Other(msg)
    }
}

impl From<&str> for PassmanError {
    fn from(msg: &str) -> Self {
        PassmanError::Other(msg.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for PassmanError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        PassmanError::Other(err.to_string())
    }
}

impl From<serde_json::Error> for PassmanError {
    fn from(err: serde_json::Error) -> Self {
        PassmanError::Other(format!("JSON error: {}", err))
    }
}

impl From<std::str::Utf8Error> for PassmanError {
    fn from(err: std::str::Utf8Error) -> Self {
        PassmanError::Other(format!("UTF-8 error: {}", err))
    }
}

impl From<csv::Error> for PassmanError {
    fn from(err: csv::Error) -> Self {
        PassmanError::Transfer(TransferError::ParseError(err.to_string()))
    }
}

impl From<std::array::TryFromSliceError> for PassmanError {
    fn from(err: std::array::TryFromSliceError) -> Self {
        PassmanError::Vault(VaultError::Corrupted(err.to_string()))
    }
}

impl From<crate::security::crypto::CryptoError> for PassmanError {
    fn from(err: crate::security::crypto::CryptoError) -> Self {
        match &err {
            crate::security::crypto::CryptoError::KeyDerivation(msg) => {
                PassmanError::Crypto(CryptoError::KeyDerivation(msg.clone()))
            }
            crate::security::crypto::CryptoError::Encryption(msg) => {
                PassmanError::Crypto(CryptoError::Encryption(msg.clone()))
            }
            crate::security::crypto::CryptoError::Decryption(msg) => {
                PassmanError::Crypto(CryptoError::Decryption(msg.clone()))
            }
        }
    }
}

/// Result type alias for Passman operations
pub type PassmanResult<T> = Result<T, PassmanError>;

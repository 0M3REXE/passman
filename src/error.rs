//! Unified Error Handling Module
//! 
//! Provides typed errors for the password manager application
//! with user-friendly error messages.

use std::fmt;
use std::io;

/// Main error type for the password manager
#[derive(Debug)]
pub enum PassmanError {
    /// Vault-related errors
    Vault(VaultError),
    /// Cryptographic errors
    Crypto(CryptoError),
    /// Authentication errors
    Auth(AuthError),
    /// Session errors
    Session(SessionError),
    /// Clipboard errors
    Clipboard(ClipboardError),
    /// Import/Export errors
    Transfer(TransferError),
    /// Configuration errors
    Config(ConfigError),
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

/// Authentication errors
#[derive(Debug, Clone)]
pub enum AuthError {
    /// Invalid master password
    InvalidPassword,
    /// Password too weak
    WeakPassword(String),
    /// Passwords don't match
    PasswordMismatch,
    /// Account is locked out
    LockedOut { remaining_secs: u64 },
    /// Too many failed attempts
    TooManyAttempts { remaining: u32 },
}

/// Session errors
#[derive(Debug, Clone)]
pub enum SessionError {
    /// Session has timed out
    TimedOut,
    /// Session is locked
    Locked,
    /// No active session
    NoSession,
    /// Session expired
    Expired,
}

/// Clipboard errors
#[derive(Debug, Clone)]
pub enum ClipboardError {
    /// Failed to access clipboard
    AccessFailed(String),
    /// Failed to set clipboard content
    SetFailed(String),
    /// Clipboard is locked
    Locked,
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

/// Configuration errors
#[derive(Debug, Clone)]
pub enum ConfigError {
    /// Config file not found
    NotFound,
    /// Invalid configuration
    Invalid(String),
    /// Failed to save config
    SaveFailed(String),
}

impl fmt::Display for PassmanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PassmanError::Vault(e) => write!(f, "{}", e),
            PassmanError::Crypto(e) => write!(f, "{}", e),
            PassmanError::Auth(e) => write!(f, "{}", e),
            PassmanError::Session(e) => write!(f, "{}", e),
            PassmanError::Clipboard(e) => write!(f, "{}", e),
            PassmanError::Transfer(e) => write!(f, "{}", e),
            PassmanError::Config(e) => write!(f, "{}", e),
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

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::InvalidPassword => write!(f, "Invalid master password."),
            AuthError::WeakPassword(msg) => write!(f, "Password is too weak: {}", msg),
            AuthError::PasswordMismatch => write!(f, "Passwords do not match."),
            AuthError::LockedOut { remaining_secs } => {
                if *remaining_secs > 60 {
                    write!(f, "Account locked. Try again in {} minutes.", remaining_secs / 60)
                } else {
                    write!(f, "Account locked. Try again in {} seconds.", remaining_secs)
                }
            }
            AuthError::TooManyAttempts { remaining } => {
                write!(f, "Too many failed attempts. {} attempts remaining.", remaining)
            }
        }
    }
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionError::TimedOut => write!(f, "Session timed out. Please unlock again."),
            SessionError::Locked => write!(f, "Session is locked. Please unlock to continue."),
            SessionError::NoSession => write!(f, "No active session. Please open a vault first."),
            SessionError::Expired => write!(f, "Session has expired. Please re-authenticate."),
        }
    }
}

impl fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClipboardError::AccessFailed(msg) => write!(f, "Cannot access clipboard: {}", msg),
            ClipboardError::SetFailed(msg) => write!(f, "Failed to copy to clipboard: {}", msg),
            ClipboardError::Locked => write!(f, "Clipboard is busy. Please try again."),
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

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::NotFound => write!(f, "Configuration file not found. Using defaults."),
            ConfigError::Invalid(msg) => write!(f, "Invalid configuration: {}", msg),
            ConfigError::SaveFailed(msg) => write!(f, "Failed to save configuration: {}", msg),
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

impl From<AuthError> for PassmanError {
    fn from(err: AuthError) -> Self {
        PassmanError::Auth(err)
    }
}

impl From<SessionError> for PassmanError {
    fn from(err: SessionError) -> Self {
        PassmanError::Session(err)
    }
}

impl From<ClipboardError> for PassmanError {
    fn from(err: ClipboardError) -> Self {
        PassmanError::Clipboard(err)
    }
}

impl From<TransferError> for PassmanError {
    fn from(err: TransferError) -> Self {
        PassmanError::Transfer(err)
    }
}

impl From<ConfigError> for PassmanError {
    fn from(err: ConfigError) -> Self {
        PassmanError::Config(err)
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

/// Result type alias for Passman operations
pub type PassmanResult<T> = Result<T, PassmanError>;

/// Helper trait for adding context to errors
pub trait ResultExt<T> {
    fn context(self, msg: &str) -> PassmanResult<T>;
}

impl<T, E: Into<PassmanError>> ResultExt<T> for Result<T, E> {
    fn context(self, msg: &str) -> PassmanResult<T> {
        self.map_err(|e| {
            let err: PassmanError = e.into();
            PassmanError::Other(format!("{}: {}", msg, err))
        })
    }
}

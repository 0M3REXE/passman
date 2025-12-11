//! Secure Types Module
//!
//! Provides secure wrappers for sensitive data with automatic memory zeroization.
//! Uses the `secrecy` crate for:
//! - Automatic zeroization on drop
//! - Debug protection (prints [REDACTED])
//! - Explicit access via `expose_secret()` trait
//!
//! This module centralizes all sensitive data handling for consistent security.

#![allow(dead_code)]

use serde::{Deserialize, Deserializer, Serialize, Serializer};

// Re-export commonly used types from secrecy
pub use secrecy::{ExposeSecret, SecretString};
#[allow(unused_imports)]
pub use secrecy::SecretBox;
pub use secrecy::zeroize::Zeroizing;
#[allow(unused_imports)]
pub use secrecy::zeroize::Zeroize;

/// A password that is automatically zeroized when dropped.
/// 
/// This type:
/// - Prevents accidental logging (Debug shows [REDACTED])
/// - Automatically clears memory when dropped
/// - Requires explicit `.expose_secret()` to access the value
/// 
/// # Example
/// ```rust
/// use crate::secure_types::SecurePassword;
/// 
/// let password = SecurePassword::new("my_secret_password");
/// // Access requires explicit call
/// let value = password.expose_secret();
/// ```
pub type SecurePassword = SecretString;

/// A TOTP secret that is automatically zeroized when dropped.
pub type SecureTotpSecret = SecretString;

/// Create a new SecretString from a String
pub fn secure_string(s: String) -> SecretString {
    SecretString::from(s)
}

/// Create a new SecretString from a &str
pub fn secure_str(s: &str) -> SecretString {
    SecretString::from(s.to_string())
}

/// Wrapper for Option<SecretString> that supports serde
#[derive(Clone)]
pub struct OptionalSecret(Option<SecretString>);

impl OptionalSecret {
    pub fn new(value: Option<String>) -> Self {
        Self(value.map(SecretString::from))
    }

    pub fn none() -> Self {
        Self(None)
    }

    pub fn some(value: String) -> Self {
        Self(Some(SecretString::from(value)))
    }

    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }

    pub fn as_ref(&self) -> Option<&SecretString> {
        self.0.as_ref()
    }

    /// Expose the secret value if present
    pub fn expose_secret(&self) -> Option<&str> {
        self.0.as_ref().map(|s| s.expose_secret())
    }

    /// Take the inner value, leaving None
    pub fn take(&mut self) -> Option<SecretString> {
        self.0.take()
    }
}

impl Default for OptionalSecret {
    fn default() -> Self {
        Self::none()
    }
}

impl std::fmt::Debug for OptionalSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(_) => write!(f, "Some([REDACTED])"),
            None => write!(f, "None"),
        }
    }
}

impl Serialize for OptionalSecret {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self.0 {
            Some(secret) => serializer.serialize_some(secret.expose_secret()),
            None => serializer.serialize_none(),
        }
    }
}

impl<'de> Deserialize<'de> for OptionalSecret {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<String> = Option::deserialize(deserializer)?;
        Ok(OptionalSecret::new(opt))
    }
}

/// Wrapper for SecretString that supports serde serialization
/// 
/// WARNING: Be careful when serializing secrets - only use when
/// the destination is trusted (e.g., encrypted vault storage)
#[derive(Clone)]
pub struct SerializableSecret(SecretString);

impl SerializableSecret {
    pub fn new(value: String) -> Self {
        Self(SecretString::from(value))
    }

    pub fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }

    pub fn inner(&self) -> &SecretString {
        &self.0
    }
}

impl From<String> for SerializableSecret {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for SerializableSecret {
    fn from(s: &str) -> Self {
        Self::new(s.to_string())
    }
}

impl Default for SerializableSecret {
    fn default() -> Self {
        Self::new(String::new())
    }
}

impl std::fmt::Debug for SerializableSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl Serialize for SerializableSecret {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.0.expose_secret())
    }
}

impl<'de> Deserialize<'de> for SerializableSecret {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(SerializableSecret::new(s))
    }
}

/// Convert Zeroizing<String> to SecretString
pub fn zeroizing_to_secret(z: Zeroizing<String>) -> SecretString {
    // Take the value out (will zeroize the Zeroizing wrapper on drop)
    SecretString::from(z.to_string())
}

/// Convert SecretString to Zeroizing<String> for compatibility
pub fn secret_to_zeroizing(s: &SecretString) -> Zeroizing<String> {
    Zeroizing::new(s.expose_secret().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_string_debug_redacted() {
        let secret = secure_str("my_password");
        let debug_output = format!("{:?}", secret);
        assert!(!debug_output.contains("my_password"));
        assert!(debug_output.contains("REDACTED") || debug_output.contains("Secret"));
    }

    #[test]
    fn test_optional_secret_debug_redacted() {
        let secret = OptionalSecret::some("my_secret".to_string());
        let debug_output = format!("{:?}", secret);
        assert!(!debug_output.contains("my_secret"));
        assert!(debug_output.contains("REDACTED"));
    }

    #[test]
    fn test_optional_secret_none() {
        let secret = OptionalSecret::none();
        assert!(secret.is_none());
        assert!(secret.expose_secret().is_none());
    }

    #[test]
    fn test_serializable_secret_expose() {
        let secret = SerializableSecret::new("test_value".to_string());
        assert_eq!(secret.expose_secret(), "test_value");
    }

    #[test]
    fn test_serializable_secret_debug_redacted() {
        let secret = SerializableSecret::new("sensitive".to_string());
        let debug_output = format!("{:?}", secret);
        assert!(!debug_output.contains("sensitive"));
        assert!(debug_output.contains("REDACTED"));
    }

    #[test]
    fn test_serializable_secret_serde_roundtrip() {
        let original = SerializableSecret::new("password123".to_string());
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: SerializableSecret = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.expose_secret(), "password123");
    }

    #[test]
    fn test_optional_secret_serde_roundtrip() {
        // Test Some case
        let original = OptionalSecret::some("totp_secret".to_string());
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: OptionalSecret = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.expose_secret(), Some("totp_secret"));

        // Test None case
        let original_none = OptionalSecret::none();
        let json_none = serde_json::to_string(&original_none).unwrap();
        let deserialized_none: OptionalSecret = serde_json::from_str(&json_none).unwrap();
        assert!(deserialized_none.is_none());
    }

    #[test]
    fn test_zeroizing_secret_conversion() {
        let zeroizing = Zeroizing::new("test".to_string());
        let secret = zeroizing_to_secret(zeroizing);
        assert_eq!(secret.expose_secret(), "test");

        let back = secret_to_zeroizing(&secret);
        assert_eq!(&*back, "test");
    }
}

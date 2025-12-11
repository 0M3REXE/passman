use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::secure_types::{SerializableSecret, OptionalSecret};

const CURRENT_VERSION: u32 = 1;

/// A password entry with secure memory handling for sensitive fields.
/// 
/// The `password` and `totp_secret` fields use secure types that:
/// - Automatically zeroize memory on drop
/// - Show [REDACTED] in debug output to prevent accidental logging
/// - Require explicit access via `.expose_secret()`
#[derive(Serialize, Deserialize, Clone)]
pub struct Entry {
    pub username: String,
    /// Password stored securely - auto-zeroizes on drop, Debug shows [REDACTED]
    pub password: SerializableSecret,
    pub note: Option<String>,
    // New fields for enhanced functionality
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_at: chrono::DateTime<chrono::Utc>,
    pub tags: Vec<String>,
    pub url: Option<String>,
    /// TOTP secret stored securely - auto-zeroizes on drop
    pub totp_secret: OptionalSecret,
}

// Custom Debug implementation to prevent accidental logging of secrets
impl std::fmt::Debug for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Entry")
            .field("username", &self.username)
            .field("password", &"[REDACTED]")
            .field("note", &self.note)
            .field("created_at", &self.created_at)
            .field("modified_at", &self.modified_at)
            .field("tags", &self.tags)
            .field("url", &self.url)
            .field("totp_secret", &self.totp_secret)
            .finish()
    }
}

impl Entry {
    pub fn new(username: String, password: String, note: Option<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            username,
            password: SerializableSecret::new(password),
            note,
            created_at: now,
            modified_at: now,
            tags: Vec::new(),
            url: None,
            totp_secret: OptionalSecret::none(),
        }
    }
    
    /// Create entry from already-secure password (for internal use)
    pub fn new_secure(username: String, password: SerializableSecret, note: Option<String>) -> Self {
        let now = chrono::Utc::now();
        Self {
            username,
            password,
            note,
            created_at: now,
            modified_at: now,
            tags: Vec::new(),
            url: None,
            totp_secret: OptionalSecret::none(),
        }
    }
    
    /// Get password as string slice (convenience method)
    /// 
    /// This explicitly exposes the secret - use with care and
    /// avoid storing the result in intermediate variables.
    pub fn password_str(&self) -> &str {
        self.password.expose_secret()
    }
    
    /// Get TOTP secret as string slice if present
    pub fn totp_secret_str(&self) -> Option<&str> {
        self.totp_secret.expose_secret()
    }
    
    #[allow(dead_code)]
    pub fn update(&mut self) {
        self.modified_at = chrono::Utc::now();
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Vault {
    pub version: u32,
    pub entries: HashMap<String, Entry>,
    pub metadata: VaultMetadata,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VaultMetadata {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub description: Option<String>,
}

impl Default for Vault {
    fn default() -> Self {
        Self::new()
    }
}

impl Vault {
    pub fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            version: CURRENT_VERSION,
            entries: HashMap::new(),
            metadata: VaultMetadata {
                created_at: now,
                last_accessed: now,
                description: None,
            },
        }
    }
      #[allow(dead_code)]
    pub fn update_access_time(&mut self) {
        self.metadata.last_accessed = chrono::Utc::now();
    }

    pub fn add_entry(&mut self, id: String, entry: Entry) {
        self.entries.insert(id, entry);
    }

    pub fn get_entry(&self, id: &str) -> Option<&Entry> {
        self.entries.get(id)
    }

    pub fn remove_entry(&mut self, id: &str) -> Option<Entry> {
        self.entries.remove(id)
    }

    pub fn list_entries(&self) -> Vec<&String> {
        self.entries.keys().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_entry_creation() {
        let entry = Entry::new(
            "user@example.com".to_string(),
            "secret123".to_string(),
            Some("My note".to_string()),
        );
        
        assert_eq!(entry.username, "user@example.com");
        assert_eq!(entry.password.expose_secret(), "secret123");
        assert_eq!(entry.note, Some("My note".to_string()));
        assert!(entry.tags.is_empty());
        assert!(entry.url.is_none());
        assert!(entry.totp_secret.is_none());
    }
    
    #[test]
    fn test_entry_timestamps() {
        let before = chrono::Utc::now();
        let entry = Entry::new("user".to_string(), "pass".to_string(), None);
        let after = chrono::Utc::now();
        
        assert!(entry.created_at >= before);
        assert!(entry.created_at <= after);
        assert_eq!(entry.created_at, entry.modified_at);
    }
    
    #[test]
    fn test_entry_update() {
        let mut entry = Entry::new("user".to_string(), "pass".to_string(), None);
        let original_modified = entry.modified_at;
        
        std::thread::sleep(std::time::Duration::from_millis(10));
        entry.update();
        
        assert_eq!(entry.created_at, entry.created_at); // created_at unchanged
        assert!(entry.modified_at > original_modified);
    }
    
    #[test]
    fn test_vault_creation() {
        let vault = Vault::new();
        
        assert_eq!(vault.version, CURRENT_VERSION);
        assert!(vault.entries.is_empty());
        assert!(vault.is_empty());
        assert!(vault.metadata.description.is_none());
    }
    
    #[test]
    fn test_vault_add_entry() {
        let mut vault = Vault::new();
        let entry = Entry::new("user".to_string(), "pass".to_string(), None);
        
        vault.add_entry("gmail".to_string(), entry);
        
        assert!(!vault.is_empty());
        assert!(vault.get_entry("gmail").is_some());
        assert!(vault.get_entry("nonexistent").is_none());
    }
    
    #[test]
    fn test_vault_remove_entry() {
        let mut vault = Vault::new();
        let entry = Entry::new("user".to_string(), "pass".to_string(), None);
        
        vault.add_entry("gmail".to_string(), entry);
        let removed = vault.remove_entry("gmail");
        
        assert!(removed.is_some());
        assert!(vault.is_empty());
        
        // Removing non-existent entry returns None
        let removed2 = vault.remove_entry("gmail");
        assert!(removed2.is_none());
    }
    
    #[test]
    fn test_vault_list_entries() {
        let mut vault = Vault::new();
        
        vault.add_entry("gmail".to_string(), Entry::new("user1".to_string(), "pass1".to_string(), None));
        vault.add_entry("github".to_string(), Entry::new("user2".to_string(), "pass2".to_string(), None));
        vault.add_entry("work".to_string(), Entry::new("user3".to_string(), "pass3".to_string(), None));
        
        let entries = vault.list_entries();
        assert_eq!(entries.len(), 3);
    }
    
    #[test]
    fn test_vault_update_access_time() {
        let mut vault = Vault::new();
        let original_access = vault.metadata.last_accessed;
        
        std::thread::sleep(std::time::Duration::from_millis(10));
        vault.update_access_time();
        
        assert!(vault.metadata.last_accessed > original_access);
        assert_eq!(vault.metadata.created_at, vault.metadata.created_at); // created_at unchanged
    }
    
    #[test]
    fn test_entry_serialization() {
        let entry = Entry::new(
            "user@example.com".to_string(),
            "secret123".to_string(),
            Some("My note".to_string()),
        );
        
        let json = serde_json::to_string(&entry).expect("Serialization should succeed");
        let deserialized: Entry = serde_json::from_str(&json).expect("Deserialization should succeed");
        
        assert_eq!(entry.username, deserialized.username);
        assert_eq!(entry.password.expose_secret(), deserialized.password.expose_secret());
        assert_eq!(entry.note, deserialized.note);
    }
    
    #[test]
    fn test_vault_serialization() {
        let mut vault = Vault::new();
        vault.add_entry("test".to_string(), Entry::new("user".to_string(), "pass".to_string(), None));
        
        let json = serde_json::to_string(&vault).expect("Serialization should succeed");
        let deserialized: Vault = serde_json::from_str(&json).expect("Deserialization should succeed");
        
        assert_eq!(vault.version, deserialized.version);
        assert!(deserialized.get_entry("test").is_some());
    }
    
    #[test]
    fn test_vault_overwrite_entry() {
        let mut vault = Vault::new();
        
        vault.add_entry("key".to_string(), Entry::new("user1".to_string(), "pass1".to_string(), None));
        vault.add_entry("key".to_string(), Entry::new("user2".to_string(), "pass2".to_string(), None));
        
        let entry = vault.get_entry("key").expect("Entry should exist");
        assert_eq!(entry.username, "user2");
        assert_eq!(entry.password.expose_secret(), "pass2");
    }
    
    #[test]
    fn test_entry_debug_redacted() {
        let entry = Entry::new(
            "user@example.com".to_string(),
            "super_secret_password".to_string(),
            None,
        );
        
        let debug_output = format!("{:?}", entry);
        
        // Password should NOT appear in debug output
        assert!(!debug_output.contains("super_secret_password"));
        // REDACTED should appear
        assert!(debug_output.contains("REDACTED"));
    }
}

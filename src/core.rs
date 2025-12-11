//! Core Library Module
//!
//! Provides a unified API for both CLI and GUI interfaces.
//! This module abstracts vault operations, authentication, and common functionality
//! to ensure consistent behavior across different frontends.

use crate::crypto::{derive_key, Key};
use crate::model::{Entry, Vault};
use crate::vault::VaultManager;
use crate::health::{PasswordHealthAnalyzer, PasswordHealth, HealthSummary, HealthReport};
use crate::import_export::ImportExportManager;
use crate::utils::{generate_password, generate_password_with_config, generate_memorable_password, analyze_password_strength, PasswordStrength, PasswordConfig};
use crate::error::{PassmanError, PassmanResult, VaultError, AuthError, CryptoError, TransferError};
use crate::config::{Config, get_config};

use argon2::password_hash::SaltString;
use zeroize::Zeroizing;
use std::path::Path;

/// Core password manager operations
/// 
/// This struct provides a unified interface for all password manager operations,
/// abstracting the underlying vault, crypto, and storage mechanisms.
pub struct PassmanCore {
    /// Currently loaded vault (if any)
    vault: Option<Vault>,
    /// Derived encryption key (if authenticated)
    key: Option<Key>,
    /// Path to the vault file
    vault_path: String,
    /// Application configuration
    config: Config,
}

impl PassmanCore {
    /// Create a new core instance with default configuration
    pub fn new() -> Self {
        let config = get_config();
        Self {
            vault: None,
            key: None,
            vault_path: config.general.default_vault.clone(),
            config: config.clone(),
        }
    }

    /// Create a new core instance with a specific vault path
    pub fn with_vault_path(vault_path: impl Into<String>) -> Self {
        let config = get_config();
        Self {
            vault: None,
            key: None,
            vault_path: vault_path.into(),
            config: config.clone(),
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Check if a vault exists at the current path
    pub fn vault_exists(&self) -> bool {
        Path::new(&self.vault_path).exists()
    }

    /// Get the vault file path
    pub fn vault_path(&self) -> &str {
        &self.vault_path
    }

    /// Set the vault file path
    pub fn set_vault_path(&mut self, path: impl Into<String>) {
        self.vault_path = path.into();
    }

    /// Check if currently authenticated (vault is unlocked)
    pub fn is_authenticated(&self) -> bool {
        self.vault.is_some() && self.key.is_some()
    }

    // ============ Vault Operations ============

    /// Initialize a new vault with the given master password
    /// 
    /// # Errors
    /// Returns error if vault already exists or password is too weak
    pub fn init_vault(&mut self, master_password: &Zeroizing<String>) -> PassmanResult<()> {
        if self.vault_exists() {
            return Err(PassmanError::Vault(VaultError::AlreadyExists(
                self.vault_path.clone()
            )));
        }

        // Validate password strength
        self.validate_master_password(master_password)?;

        VaultManager::init(master_password, Some(&self.vault_path))
            .map_err(|e| PassmanError::Vault(VaultError::WriteError(e.to_string())))?;

        // Auto-login after init
        self.unlock(master_password)?;

        log::info!("Vault initialized at {}", self.vault_path);
        Ok(())
    }

    /// Unlock the vault with the master password
    /// 
    /// # Errors
    /// Returns error if vault doesn't exist or password is incorrect
    pub fn unlock(&mut self, master_password: &Zeroizing<String>) -> PassmanResult<()> {
        if !self.vault_exists() {
            return Err(PassmanError::Vault(VaultError::NotFound(
                self.vault_path.clone()
            )));
        }

        let vault = VaultManager::load(master_password, Some(&self.vault_path))
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("decryption") || msg.contains("authentication") || msg.contains("HMAC") {
                    PassmanError::Auth(AuthError::InvalidPassword)
                } else {
                    PassmanError::Vault(VaultError::ReadError(msg))
                }
            })?;

        // Derive key for future saves
        let salt = SaltString::generate(&mut rand::thread_rng());
        let key = derive_key(master_password.as_str(), &salt)
            .map_err(|e| PassmanError::Crypto(CryptoError::KeyDerivation(e.to_string())))?;

        self.vault = Some(vault);
        self.key = Some(key);

        log::info!("Vault unlocked successfully");
        Ok(())
    }

    /// Lock the vault (clear sensitive data from memory)
    pub fn lock(&mut self) {
        self.vault = None;
        self.key = None;
        log::info!("Vault locked");
    }

    /// Save the current vault state
    /// 
    /// # Errors
    /// Returns error if vault is not unlocked
    pub fn save(&self, master_password: &Zeroizing<String>) -> PassmanResult<()> {
        let vault = self.vault.as_ref()
            .ok_or_else(|| PassmanError::Vault(VaultError::ReadError("No vault loaded".to_string())))?;

        VaultManager::save(vault, master_password, Some(&self.vault_path))
            .map_err(|e| PassmanError::Vault(VaultError::WriteError(e.to_string())))?;

        log::debug!("Vault saved");
        Ok(())
    }

    /// Change the master password
    /// 
    /// # Errors
    /// Returns error if current password is incorrect or new password is too weak
    pub fn change_password(
        &mut self,
        current_password: &Zeroizing<String>,
        new_password: &Zeroizing<String>,
    ) -> PassmanResult<()> {
        // Verify current password by loading vault
        let vault = VaultManager::load(current_password, Some(&self.vault_path))
            .map_err(|_| PassmanError::Auth(AuthError::InvalidPassword))?;

        // Validate new password
        self.validate_master_password(new_password)?;

        // Save with new password
        VaultManager::save(&vault, new_password, Some(&self.vault_path))
            .map_err(|e| PassmanError::Vault(VaultError::WriteError(e.to_string())))?;

        // Update internal state
        self.vault = Some(vault);

        log::info!("Master password changed successfully");
        Ok(())
    }

    // ============ Entry Operations ============

    /// Get a reference to the current vault
    pub fn vault(&self) -> Option<&Vault> {
        self.vault.as_ref()
    }

    /// Get a mutable reference to the current vault
    pub fn vault_mut(&mut self) -> Option<&mut Vault> {
        self.vault.as_mut()
    }

    /// Get an entry by ID
    pub fn get_entry(&self, id: &str) -> Option<&Entry> {
        self.vault.as_ref()?.get_entry(id)
    }

    /// List all entry IDs
    pub fn list_entries(&self) -> Vec<String> {
        self.vault.as_ref()
            .map(|v| v.list_entries().into_iter().cloned().collect())
            .unwrap_or_default()
    }

    /// List all entries with their data
    pub fn list_entries_with_data(&self) -> Vec<(String, Entry)> {
        self.vault.as_ref()
            .map(|v| {
                v.list_entries()
                    .into_iter()
                    .filter_map(|id| v.get_entry(id).map(|e| (id.clone(), e.clone())))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Add a new entry
    /// 
    /// # Errors
    /// Returns error if vault is locked or entry already exists
    pub fn add_entry(&mut self, id: impl Into<String>, entry: Entry) -> PassmanResult<()> {
        let id = id.into();
        let vault = self.vault.as_mut()
            .ok_or_else(|| PassmanError::Vault(VaultError::ReadError("Vault is locked".to_string())))?;

        if vault.get_entry(&id).is_some() {
            return Err(PassmanError::Vault(VaultError::EntryExists(id)));
        }

        vault.add_entry(id.clone(), entry);
        log::debug!("Entry added: {}", id);
        Ok(())
    }

    /// Update an existing entry
    /// 
    /// # Errors
    /// Returns error if vault is locked or entry doesn't exist
    pub fn update_entry(&mut self, id: &str, entry: Entry) -> PassmanResult<()> {
        let vault = self.vault.as_mut()
            .ok_or_else(|| PassmanError::Vault(VaultError::ReadError("Vault is locked".to_string())))?;

        if vault.get_entry(id).is_none() {
            return Err(PassmanError::Vault(VaultError::EntryNotFound(id.to_string())));
        }

        vault.add_entry(id.to_string(), entry);
        log::debug!("Entry updated: {}", id);
        Ok(())
    }

    /// Remove an entry
    /// 
    /// # Errors
    /// Returns error if vault is locked or entry doesn't exist
    pub fn remove_entry(&mut self, id: &str) -> PassmanResult<Entry> {
        let vault = self.vault.as_mut()
            .ok_or_else(|| PassmanError::Vault(VaultError::ReadError("Vault is locked".to_string())))?;

        vault.remove_entry(id)
            .ok_or_else(|| PassmanError::Vault(VaultError::EntryNotFound(id.to_string())))
    }

    /// Search entries by pattern (matches ID or username)
    pub fn search_entries(&self, pattern: &str) -> Vec<(String, Entry)> {
        let pattern_lower = pattern.to_lowercase();
        self.list_entries_with_data()
            .into_iter()
            .filter(|(id, entry)| {
                id.to_lowercase().contains(&pattern_lower) ||
                entry.username.to_lowercase().contains(&pattern_lower)
            })
            .collect()
    }

    /// Check if vault is empty
    pub fn is_empty(&self) -> bool {
        self.vault.as_ref().map_or(true, |v| v.is_empty())
    }

    /// Get entry count
    pub fn entry_count(&self) -> usize {
        self.vault.as_ref().map_or(0, |v| v.entries.len())
    }

    // ============ Password Generation ============

    /// Generate a random password with default settings
    pub fn generate_password(&self, length: usize) -> String {
        generate_password(length)
    }

    /// Generate a random password with custom configuration
    pub fn generate_password_configured(&self, length: usize, config: &PasswordConfig) -> String {
        generate_password_with_config(length, config)
    }

    /// Generate a memorable password (diceware-style)
    pub fn generate_memorable_password(&self, word_count: usize) -> String {
        generate_memorable_password(word_count)
    }

    /// Analyze password strength
    pub fn analyze_password(&self, password: &str) -> (PasswordStrength, Vec<String>) {
        analyze_password_strength(password)
    }

    // ============ Health Analysis ============

    /// Analyze the health of all passwords in the vault
    pub fn analyze_health(&self) -> Option<(Vec<HealthReport>, HealthSummary)> {
        let vault = self.vault.as_ref()?;
        let analyzer = PasswordHealthAnalyzer::new();
        let reports = analyzer.analyze_vault(vault);
        let summary = analyzer.generate_summary(&reports);
        Some((reports, summary))
    }

    /// Get detailed health analysis for a specific entry
    pub fn analyze_entry_health(&self, id: &str) -> Option<HealthReport> {
        let entry = self.get_entry(id)?;
        let analyzer = PasswordHealthAnalyzer::new();
        Some(analyzer.analyze_entry(id, entry))
    }

    /// Get entries with weak passwords
    pub fn get_weak_passwords(&self) -> Vec<HealthReport> {
        let vault = match self.vault.as_ref() {
            Some(v) => v,
            None => return Vec::new(),
        };

        let analyzer = PasswordHealthAnalyzer::new();
        analyzer.analyze_vault(vault)
            .into_iter()
            .filter(|r| matches!(r.health, PasswordHealth::Critical { .. } | PasswordHealth::Warning { .. }))
            .collect()
    }

    /// Get entries with reused passwords (same password across entries)
    pub fn get_reused_passwords(&self) -> Vec<Vec<String>> {
        let vault = match self.vault.as_ref() {
            Some(v) => v,
            None => return Vec::new(),
        };

        use std::collections::HashMap;
        let mut password_map: HashMap<&str, Vec<String>> = HashMap::new();

        for id in vault.list_entries() {
            if let Some(entry) = vault.get_entry(id) {
                password_map
                    .entry(&entry.password)
                    .or_default()
                    .push(id.clone());
            }
        }

        password_map
            .into_values()
            .filter(|ids| ids.len() > 1)
            .collect()
    }

    // ============ Import/Export ============

    /// Export vault to JSON format
    pub fn export_json(&self, file_path: &str) -> PassmanResult<()> {
        let vault = self.vault.as_ref()
            .ok_or_else(|| PassmanError::Transfer(TransferError::InvalidData("Vault is locked".to_string())))?;

        ImportExportManager::export_json(vault, file_path)
            .map_err(|e| PassmanError::Transfer(TransferError::InvalidData(e.to_string())))
    }

    /// Export vault to CSV format (WARNING: plaintext)
    pub fn export_csv(&self, file_path: &str) -> PassmanResult<()> {
        let vault = self.vault.as_ref()
            .ok_or_else(|| PassmanError::Transfer(TransferError::InvalidData("Vault is locked".to_string())))?;

        ImportExportManager::export_csv(vault, file_path)
            .map_err(|e| PassmanError::Transfer(TransferError::InvalidData(e.to_string())))
    }

    /// Import entries from JSON file
    pub fn import_json(&mut self, file_path: &str, master_password: &Zeroizing<String>, merge: bool) -> PassmanResult<()> {
        ImportExportManager::import_json(file_path, master_password, Some(&self.vault_path), merge)
            .map_err(|e| PassmanError::Transfer(TransferError::ParseError(e.to_string())))?;

        // Reload vault after import
        self.unlock(master_password)?;

        Ok(())
    }

    /// Import entries from CSV file
    pub fn import_csv(&mut self, file_path: &str, master_password: &Zeroizing<String>, merge: bool) -> PassmanResult<()> {
        ImportExportManager::import_csv(file_path, master_password, Some(&self.vault_path), merge)
            .map_err(|e| PassmanError::Transfer(TransferError::ParseError(e.to_string())))?;

        // Reload vault after import
        self.unlock(master_password)?;

        Ok(())
    }

    /// Create a backup of the current vault
    pub fn create_backup(&self) -> PassmanResult<String> {
        VaultManager::create_backup(Some(&self.vault_path))
            .map_err(|e| PassmanError::Transfer(TransferError::InvalidData(e.to_string())))
    }


    // ============ Validation Helpers ============

    /// Validate master password meets minimum requirements
    fn validate_master_password(&self, password: &Zeroizing<String>) -> PassmanResult<()> {
        if password.len() < self.config.security.min_password_length {
            return Err(PassmanError::Auth(AuthError::WeakPassword(format!(
                "Password must be at least {} characters",
                self.config.security.min_password_length
            ))));
        }

        let (strength, _) = analyze_password_strength(password.as_str());
        match strength {
            PasswordStrength::VeryWeak | PasswordStrength::Weak => {
                Err(PassmanError::Auth(AuthError::WeakPassword(
                    "Master password is too weak. Use a stronger password.".to_string()
                )))
            }
            _ => Ok(())
        }
    }

    /// Create a new entry with default values
    pub fn create_entry(
        &self,
        username: impl Into<String>,
        password: impl Into<String>,
        note: Option<String>,
    ) -> Entry {
        Entry::new(username.into(), password.into(), note)
    }
}

impl Default for PassmanCore {
    fn default() -> Self {
        Self::new()
    }
}

// ============ Builder Pattern for Entry Creation ============

/// Builder for creating entries with optional fields
pub struct EntryBuilder {
    username: String,
    password: String,
    note: Option<String>,
    url: Option<String>,
    tags: Vec<String>,
}

impl EntryBuilder {
    pub fn new(username: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: String::new(),
            note: None,
            url: None,
            tags: Vec::new(),
        }
    }

    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = password.into();
        self
    }

    pub fn generate_password(mut self, length: usize) -> Self {
        self.password = generate_password(length);
        self
    }

    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn build(self) -> Entry {
        let mut entry = Entry::new(self.username, self.password, self.note);
        if let Some(url) = self.url {
            entry.url = Some(url);
        }
        entry.tags = self.tags;
        entry
    }
}

// ============ Unit Tests ============

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_creation() {
        let core = PassmanCore::new();
        assert!(!core.is_authenticated());
        assert!(core.vault().is_none());
    }

    #[test]
    fn test_core_with_vault_path() {
        let core = PassmanCore::with_vault_path("/tmp/test_vault.dat");
        assert_eq!(core.vault_path(), "/tmp/test_vault.dat");
    }

    #[test]
    fn test_entry_builder() {
        let entry = EntryBuilder::new("user@example.com")
            .password("SecurePass123!")
            .note("Test note")
            .url("https://example.com")
            .tag("work")
            .tag("important")
            .build();

        assert_eq!(entry.username, "user@example.com");
        assert_eq!(entry.password, "SecurePass123!");
        assert_eq!(entry.note, Some("Test note".to_string()));
        assert_eq!(entry.url, Some("https://example.com".to_string()));
        assert_eq!(entry.tags.len(), 2);
    }

    #[test]
    fn test_entry_builder_generate_password() {
        let entry = EntryBuilder::new("user@example.com")
            .generate_password(20)
            .build();

        assert_eq!(entry.password.len(), 20);
    }

    #[test]
    fn test_password_generation() {
        let core = PassmanCore::new();
        
        let password = core.generate_password(16);
        assert_eq!(password.len(), 16);
        
        let memorable = core.generate_memorable_password(4);
        assert!(!memorable.is_empty());
    }

    #[test]
    fn test_password_analysis() {
        let core = PassmanCore::new();
        
        let (strength, _) = core.analyze_password("weak");
        assert!(matches!(strength, PasswordStrength::VeryWeak | PasswordStrength::Weak));
        
        let (strength, _) = core.analyze_password("Str0ng!P@ssw0rd#2024");
        assert!(matches!(strength, PasswordStrength::Good | PasswordStrength::Strong));
    }

    #[test]
    fn test_create_entry() {
        let core = PassmanCore::new();
        let entry = core.create_entry("user", "pass", Some("note".to_string()));
        
        assert_eq!(entry.username, "user");
        assert_eq!(entry.password, "pass");
        assert_eq!(entry.note, Some("note".to_string()));
    }
}


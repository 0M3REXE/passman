//! Vault Management Module
//! 
//! Provides secure storage for password entries with:
//! - AES-256-GCM encryption
//! - Argon2id key derivation
//! - HMAC-SHA256 integrity verification
//! - Atomic file writes to prevent corruption

use crate::crypto::{derive_key, encrypt_data, decrypt_data, Key};
use crate::model::Vault;
use argon2::password_hash::SaltString;
use std::fs::{self, File, read_dir};
use std::io::{Write, Read};
use std::path::Path;
use zeroize::Zeroizing;
use std::time::{Duration, Instant};
use std::thread;
use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};

type HmacSha256 = Hmac<Sha256>;

/// Default vault file name
const DEFAULT_VAULT_FILE: &str = "vault.dat";

/// Vault file format version
const VAULT_FORMAT_VERSION: u8 = 2;

/// Magic bytes to identify vault files
const VAULT_MAGIC: &[u8; 4] = b"PMAN";

/// Vault file header structure
#[derive(Debug)]
struct VaultHeader {
    magic: [u8; 4],
    version: u8,
    salt_len: u32,
}

impl VaultHeader {
    fn new(salt_len: u32) -> Self {
        Self {
            magic: *VAULT_MAGIC,
            version: VAULT_FORMAT_VERSION,
            salt_len,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(9);
        bytes.extend_from_slice(&self.magic);
        bytes.push(self.version);
        bytes.extend_from_slice(&self.salt_len.to_le_bytes());
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 9 {
            return None;
        }

        let magic: [u8; 4] = bytes[0..4].try_into().ok()?;
        if &magic != VAULT_MAGIC {
            return None; // Legacy format
        }

        let version = bytes[4];
        let salt_len = u32::from_le_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]);

        Some(Self { magic, version, salt_len })
    }
}

pub struct VaultManager;

impl VaultManager {
    /// Get the vault file path
    fn get_vault_path(vault_file: Option<&str>) -> &str {
        vault_file.unwrap_or(DEFAULT_VAULT_FILE)
    }

    /// Generate HMAC for vault data
    fn generate_hmac(key: &Key, data: &[u8]) -> Vec<u8> {
        let mut mac = HmacSha256::new_from_slice(key.as_ref())
            .expect("HMAC can take key of any size");
        mac.update(data);
        mac.finalize().into_bytes().to_vec()
    }

    /// Verify HMAC for vault data
    fn verify_hmac(key: &Key, data: &[u8], expected_hmac: &[u8]) -> bool {
        let mut mac = HmacSha256::new_from_slice(key.as_ref())
            .expect("HMAC can take key of any size");
        mac.update(data);
        mac.verify_slice(expected_hmac).is_ok()
    }

    /// Write data atomically (write to temp file, then rename)
    fn atomic_write(path: &str, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let temp_path = format!("{}.tmp", path);
        let backup_path = format!("{}.bak", path);

        // Write to temporary file
        {
            let mut file = File::create(&temp_path)?;
            file.write_all(data)?;
            file.sync_all()?;
        }

        // Create backup of existing file if it exists
        if Path::new(path).exists() {
            let _ = fs::remove_file(&backup_path);
            fs::rename(path, &backup_path)?;
        }

        // Rename temp to final
        fs::rename(&temp_path, path)?;

        Ok(())
    }

    /// Initialize a new encrypted vault with master password
    pub fn init(master_password: &Zeroizing<String>, vault_file: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        let vault_path = Self::get_vault_path(vault_file);
        
        if Path::new(vault_path).exists() {
            return Err(format!("Vault '{}' already exists! Remove it to reset.", vault_path).into());
        }

        let salt = SaltString::generate(&mut rand::thread_rng());
        let key = derive_key(master_password.as_str(), &salt)?;

        let vault = Vault::new();
        let serialized = serde_json::to_vec(&vault)?;

        let (ciphertext, nonce) = encrypt_data(&key, &serialized)?;

        // Build vault file (v2 format with HMAC)
        let salt_bytes = salt.as_str().as_bytes();
        let header = VaultHeader::new(salt_bytes.len() as u32);
        
        // HMAC covers nonce + ciphertext
        let mut hmac_data = Vec::new();
        hmac_data.extend_from_slice(&nonce);
        hmac_data.extend_from_slice(&ciphertext);
        let hmac = Self::generate_hmac(&key, &hmac_data);

        // Assemble file: [header(9)][salt][nonce(12)][hmac(32)][ciphertext]
        let mut file_data = Vec::new();
        file_data.extend_from_slice(&header.to_bytes());
        file_data.extend_from_slice(salt_bytes);
        file_data.extend_from_slice(&nonce);
        file_data.extend_from_slice(&hmac);
        file_data.extend_from_slice(&ciphertext);

        Self::atomic_write(vault_path, &file_data)?;

        log::info!("Vault initialized: {}", vault_path);
        Ok(())
    }    /// Load and decrypt vault with master password
    pub fn load(master_password: &Zeroizing<String>, vault_file: Option<&str>) -> Result<Vault, Box<dyn std::error::Error>> {
        let vault_path = Self::get_vault_path(vault_file);
        
        if !Path::new(vault_path).exists() {
            return Err(format!("Vault '{}' not found! Run 'passman init' first.", vault_path).into());
        }

        let mut file = File::open(vault_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // Try v2 format first
        if let Some(header) = VaultHeader::from_bytes(&buffer) {
            // V2 format: [header(9)][salt][nonce(12)][hmac(32)][ciphertext]
            let mut offset = 9;
            
            // Read salt
            let salt_end = offset + header.salt_len as usize;
            if buffer.len() < salt_end + 44 { // 12 (nonce) + 32 (hmac)
                return Err("Vault file corrupted: too short".into());
            }
            let salt_str = std::str::from_utf8(&buffer[offset..salt_end])?;
            let salt = SaltString::from_b64(salt_str)
                .map_err(|e| format!("Salt parsing error: {}", e))?;
            offset = salt_end;

            // Read nonce
            let nonce: [u8; 12] = buffer[offset..offset + 12].try_into()?;
            offset += 12;

            // Read HMAC
            let stored_hmac = &buffer[offset..offset + 32];
            offset += 32;

            // Read ciphertext
            let ciphertext = &buffer[offset..];

            // Derive key
            let key = derive_key(master_password.as_str(), &salt)?;

            // Verify HMAC
            let mut hmac_data = Vec::new();
            hmac_data.extend_from_slice(&nonce);
            hmac_data.extend_from_slice(ciphertext);
            
            if !Self::verify_hmac(&key, &hmac_data, stored_hmac) {
                return Err("Vault integrity check failed. Wrong password or tampered file.".into());
            }

            // Decrypt
            let plaintext = decrypt_data(&key, ciphertext, &nonce)?;
            let vault: Vault = serde_json::from_slice(&plaintext)?;
            
            log::info!("Vault loaded (v2 format): {}", vault_path);
            return Ok(vault);
        }

        // Legacy format: [salt_len(4)][salt][nonce(12)][ciphertext]
        Self::load_legacy(master_password, vault_path, &buffer)
    }

    /// Load legacy format vault (backward compatibility)
    fn load_legacy(
        master_password: &Zeroizing<String>,
        vault_path: &str,
        buffer: &[u8],
    ) -> Result<Vault, Box<dyn std::error::Error>> {
        let mut offset = 0;
        
        // Read salt length (4 bytes)
        if buffer.len() < 4 {
            return Err("Vault file too short".into());
        }
        let salt_len = u32::from_le_bytes([
            buffer[offset], buffer[offset + 1], 
            buffer[offset + 2], buffer[offset + 3]
        ]) as usize;
        offset += 4;

        if salt_len > 1000 || buffer.len() < offset + salt_len + 12 {
            return Err("Invalid salt length in vault file".into());
        }

        // Read salt
        let salt_str = std::str::from_utf8(&buffer[offset..offset + salt_len])?;
        let salt = SaltString::from_b64(salt_str)
            .map_err(|e| format!("Salt parsing error: {}", e))?;
        offset += salt_len;

        // Read nonce (12 bytes)
        let nonce: [u8; 12] = buffer[offset..offset + 12].try_into()?;
        offset += 12;

        // Read ciphertext
        let ciphertext = &buffer[offset..];

        // Derive key and decrypt
        let key = derive_key(master_password.as_str(), &salt)?;
        let plaintext = decrypt_data(&key, ciphertext, &nonce)?;
        
        let vault: Vault = serde_json::from_slice(&plaintext)?;
        
        log::warn!("Loaded legacy vault format (v1): {}. Re-save to upgrade to v2.", vault_path);
        Ok(vault)
    }    /// Save encrypted vault (v2 format with HMAC and atomic write)
    pub fn save(vault: &Vault, master_password: &Zeroizing<String>, vault_file: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        let vault_path = Self::get_vault_path(vault_file);
        
        // Read existing file to get salt
        let salt = if Path::new(vault_path).exists() {
            let mut file = File::open(vault_path)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;

            // Try v2 format first
            if let Some(header) = VaultHeader::from_bytes(&buffer) {
                let salt_str = std::str::from_utf8(&buffer[9..9 + header.salt_len as usize])?;
                SaltString::from_b64(salt_str)
                    .map_err(|e| format!("Salt parsing error: {}", e))?
            } else {
                // Legacy format
                let salt_len = u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]) as usize;
                let salt_str = std::str::from_utf8(&buffer[4..4 + salt_len])?;
                SaltString::from_b64(salt_str)
                    .map_err(|e| format!("Salt parsing error: {}", e))?
            }
        } else {
            SaltString::generate(&mut rand::thread_rng())
        };

        // Derive key
        let key = derive_key(master_password.as_str(), &salt)?;

        // Serialize and encrypt vault
        let serialized = serde_json::to_vec(vault)?;
        let (ciphertext, nonce) = encrypt_data(&key, &serialized)?;

        // Build v2 format file
        let salt_bytes = salt.as_str().as_bytes();
        let header = VaultHeader::new(salt_bytes.len() as u32);
        
        // Generate HMAC
        let mut hmac_data = Vec::new();
        hmac_data.extend_from_slice(&nonce);
        hmac_data.extend_from_slice(&ciphertext);
        let hmac = Self::generate_hmac(&key, &hmac_data);

        // Assemble file
        let mut file_data = Vec::new();
        file_data.extend_from_slice(&header.to_bytes());
        file_data.extend_from_slice(salt_bytes);
        file_data.extend_from_slice(&nonce);
        file_data.extend_from_slice(&hmac);
        file_data.extend_from_slice(&ciphertext);

        // Atomic write
        Self::atomic_write(vault_path, &file_data)?;

        log::info!("Vault saved: {}", vault_path);
        Ok(())
    }

    /// Check if vault exists
    pub fn exists(vault_file: Option<&str>) -> bool {
        let vault_path = Self::get_vault_path(vault_file);
        Path::new(vault_path).exists()
    }    /// List all vault files in current directory
    #[allow(dead_code)]
    pub fn list_vaults() -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut vaults = Vec::new();
        
        for entry in read_dir(".")? {
            let entry = entry?;
            let path = entry.path();
            if let Some(name) = path.file_name() {
                if let Some(name_str) = name.to_str() {
                    if name_str.ends_with(".dat") {
                        vaults.push(name_str.to_string());
                    }
                }
            }
        }
        
        vaults.sort();
        Ok(vaults)
    }    /// Verify vault integrity using HMAC (requires password)
    pub fn verify_integrity(master_password: &Zeroizing<String>, vault_file: Option<&str>) -> Result<bool, Box<dyn std::error::Error>> {
        let vault_path = Self::get_vault_path(vault_file);
        
        if !Path::new(vault_path).exists() {
            return Err("Vault file not found".into());
        }

        let mut file = File::open(vault_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // Check for v2 format
        if let Some(header) = VaultHeader::from_bytes(&buffer) {
            let salt_str = std::str::from_utf8(&buffer[9..9 + header.salt_len as usize])?;
            let salt = SaltString::from_b64(salt_str)
                .map_err(|e| format!("Salt parsing error: {}", e))?;
            
            let key = derive_key(master_password.as_str(), &salt)?;
            
            let offset = 9 + header.salt_len as usize;
            let nonce = &buffer[offset..offset + 12];
            let stored_hmac = &buffer[offset + 12..offset + 44];
            let ciphertext = &buffer[offset + 44..];
            
            let mut hmac_data = Vec::new();
            hmac_data.extend_from_slice(nonce);
            hmac_data.extend_from_slice(ciphertext);
            
            let valid = Self::verify_hmac(&key, &hmac_data, stored_hmac);
            
            if valid {
                log::info!("Vault integrity verified (HMAC): {}", vault_path);
            } else {
                log::error!("Vault integrity check FAILED: {}", vault_path);
            }
            
            return Ok(valid);
        }

        // Legacy format - no HMAC verification available
        log::warn!("Legacy vault format does not support HMAC integrity verification");
        
        // Fall back to SHA-256 checksum logging
        let mut hasher = Sha256::new();
        hasher.update(&buffer);
        let current_hash = hasher.finalize();
        log::info!("Vault SHA-256: {:x}", current_hash);
        
        Ok(true)
    }

    /// Create a backup of the vault with timestamp
    pub fn create_backup(vault_file: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
        let vault_path = Self::get_vault_path(vault_file);
        
        if !Path::new(vault_path).exists() {
            return Err("Vault file not found".into());
        }

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("{}.backup.{}", vault_path, timestamp);
        
        fs::copy(vault_path, &backup_name)?;
        log::info!("Vault backup created: {}", backup_name);
        
        Ok(backup_name)
    }

    /// Change master password (re-encrypts the vault with new password)
    pub fn change_password(
        old_password: &Zeroizing<String>,
        new_password: &Zeroizing<String>,
        vault_file: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let vault_path = Self::get_vault_path(vault_file);
        
        // Create backup first
        let backup = Self::create_backup(vault_file)?;
        log::info!("Created backup before password change: {}", backup);

        // Load vault with old password
        let vault = Self::load(old_password, vault_file)?;

        // Generate new salt for new password
        let new_salt = SaltString::generate(&mut rand::thread_rng());
        let new_key = derive_key(new_password.as_str(), &new_salt)?;

        // Re-encrypt vault
        let serialized = serde_json::to_vec(&vault)?;
        let (ciphertext, nonce) = encrypt_data(&new_key, &serialized)?;

        // Build new vault file (v2 format)
        let salt_bytes = new_salt.as_str().as_bytes();
        let header = VaultHeader::new(salt_bytes.len() as u32);
        
        let mut hmac_data = Vec::new();
        hmac_data.extend_from_slice(&nonce);
        hmac_data.extend_from_slice(&ciphertext);
        let hmac = Self::generate_hmac(&new_key, &hmac_data);

        let mut file_data = Vec::new();
        file_data.extend_from_slice(&header.to_bytes());
        file_data.extend_from_slice(salt_bytes);
        file_data.extend_from_slice(&nonce);
        file_data.extend_from_slice(&hmac);
        file_data.extend_from_slice(&ciphertext);

        Self::atomic_write(vault_path, &file_data)?;

        log::info!("Master password changed successfully: {}", vault_path);
        Ok(())
    }

    /// Delete a vault file
    pub fn delete(vault_file: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        let vault_path = Self::get_vault_path(vault_file);
        if Path::new(vault_path).exists() {
            fs::remove_file(vault_path)?;
            log::info!("Vault deleted: {}", vault_path);
        }
        Ok(())
    }
}

/// Security manager for handling authentication delays and security policies
#[allow(dead_code)]
pub struct SecurityManager {
    failed_attempts: u32,
    last_attempt: Option<Instant>,
    lockout_until: Option<Instant>,
}

impl SecurityManager {
    pub fn new() -> Self {
        Self {
            failed_attempts: 0,
            last_attempt: None,
            lockout_until: None,
        }
    }

    /// Check if authentication is currently locked out
    #[allow(dead_code)]
    pub fn is_locked_out(&self) -> bool {
        if let Some(lockout_time) = self.lockout_until {
            Instant::now() < lockout_time
        } else {
            false
        }
    }

    /// Get remaining lockout time in seconds
    #[allow(dead_code)]
    pub fn remaining_lockout_time(&self) -> Option<u64> {
        if let Some(lockout_time) = self.lockout_until {
            let now = Instant::now();
            if now < lockout_time {
                Some((lockout_time - now).as_secs())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get remaining lockout seconds (convenience method)
    pub fn lockout_remaining_secs(&self) -> u64 {
        self.remaining_lockout_time().unwrap_or(0)
    }

    /// Get remaining login attempts before lockout
    pub fn remaining_attempts(&self) -> u32 {
        5u32.saturating_sub(self.failed_attempts)
    }

    /// Record a failed authentication attempt
    #[allow(dead_code)]
    pub fn record_failed_attempt(&mut self) {
        self.failed_attempts += 1;
        self.last_attempt = Some(Instant::now());

        // Implement exponential backoff
        let delay = match self.failed_attempts {
            1..=2 => Duration::from_secs(1),
            3..=4 => Duration::from_secs(2),
            5..=6 => Duration::from_secs(5),
            7..=8 => Duration::from_secs(10),
            _ => Duration::from_secs(30),
        };

        // Lock out for longer periods after many attempts
        if self.failed_attempts >= 5 {
            self.lockout_until = Some(Instant::now() + delay);
        }

        // Add immediate delay to slow down brute force
        thread::sleep(delay);
    }

    /// Record a successful authentication attempt
    #[allow(dead_code)]
    pub fn record_successful_attempt(&mut self) {
        self.failed_attempts = 0;
        self.last_attempt = None;
        self.lockout_until = None;
    }

    /// Alias for record_successful_attempt (for API consistency)
    pub fn record_successful_login(&mut self) {
        self.record_successful_attempt();
    }

    /// Clear all security state (for testing purposes)
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.failed_attempts = 0;
        self.last_attempt = None;
        self.lockout_until = None;
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new()
    }
}

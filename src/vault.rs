use crate::crypto::{derive_key, encrypt_data, decrypt_data};
use crate::model::Vault;
use argon2::password_hash::SaltString;
use std::fs::{File, read_dir};
use std::io::{Write, Read};
use std::path::Path;
use zeroize::Zeroizing;
use std::time::{Duration, Instant};
use std::thread;
use sha2::{Sha256, Digest};

const DEFAULT_VAULT_FILE: &str = "vault.dat";

pub struct VaultManager;

impl VaultManager {
    /// Get the vault file path
    fn get_vault_path(vault_file: Option<&str>) -> &str {
        vault_file.unwrap_or(DEFAULT_VAULT_FILE)
    }    /// Initialize a new encrypted vault with master password
    pub fn init(master_password: &Zeroizing<String>, vault_file: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        let vault_path = Self::get_vault_path(vault_file);
        
        if Path::new(vault_path).exists() {
            return Err(format!("Vault '{}' already exists! Remove it to reset.", vault_path).into());
        }        let salt = SaltString::generate(&mut rand::thread_rng());
        let key = derive_key(master_password.as_str(), &salt)?;

        let vault = Vault::new();
        let serialized = serde_json::to_vec(&vault)?;

        let (ciphertext, nonce) = encrypt_data(&key, &serialized)?;

        // Save: [salt_len][salt][nonce][ciphertext]
        let mut file = File::create(vault_path)?;
        let salt_bytes = salt.as_str().as_bytes();
        file.write_all(&(salt_bytes.len() as u32).to_le_bytes())?; // 4 bytes for salt length
        file.write_all(salt_bytes)?;
        file.write_all(&nonce)?;
        file.write_all(&ciphertext)?;

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

        // Parse file format: [salt_len][salt][nonce][ciphertext]
        let mut offset = 0;
        
        // Read salt length (4 bytes)
        let salt_len = u32::from_le_bytes([
            buffer[offset], buffer[offset + 1], 
            buffer[offset + 2], buffer[offset + 3]
        ]) as usize;
        offset += 4;        // Read salt
        let salt_str = std::str::from_utf8(&buffer[offset..offset + salt_len])?;
        let salt = SaltString::from_b64(salt_str).map_err(|e| format!("Salt parsing error: {}", e))?;
        offset += salt_len;

        // Read nonce (12 bytes)
        let nonce: [u8; 12] = buffer[offset..offset + 12].try_into()?;
        offset += 12;

        // Read ciphertext (rest of the file)
        let ciphertext = &buffer[offset..];        // Derive key and decrypt
        let key = derive_key(master_password.as_str(), &salt)?;
        let plaintext = decrypt_data(&key, ciphertext, &nonce)?;
        
        let vault: Vault = serde_json::from_slice(&plaintext)?;
        Ok(vault)
    }    /// Save encrypted vault
    pub fn save(vault: &Vault, master_password: &Zeroizing<String>, vault_file: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        let vault_path = Self::get_vault_path(vault_file);
        
        // Read existing salt from file
        let mut file = File::open(vault_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let mut offset = 0;
        
        // Read salt length (4 bytes)
        let salt_len = u32::from_le_bytes([
            buffer[offset], buffer[offset + 1], 
            buffer[offset + 2], buffer[offset + 3]
        ]) as usize;
        offset += 4;        // Read salt
        let salt_str = std::str::from_utf8(&buffer[offset..offset + salt_len])?;
        let salt = SaltString::from_b64(salt_str).map_err(|e| format!("Salt parsing error: {}", e))?;        // Derive key
        let key = derive_key(master_password.as_str(), &salt)?;

        // Serialize and encrypt vault
        let serialized = serde_json::to_vec(vault)?;
        let (ciphertext, nonce) = encrypt_data(&key, &serialized)?;

        // Write back to file
        let mut file = File::create(vault_path)?;
        let salt_bytes = salt.as_str().as_bytes();
        file.write_all(&(salt_bytes.len() as u32).to_le_bytes())?;
        file.write_all(salt_bytes)?;
        file.write_all(&nonce)?;
        file.write_all(&ciphertext)?;

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
    }    /// Verify vault integrity using SHA-256 checksum
    #[allow(dead_code)]
    pub fn verify_integrity(vault_file: Option<&str>) -> Result<bool, Box<dyn std::error::Error>> {
        let vault_path = Self::get_vault_path(vault_file);
        
        if !Path::new(vault_path).exists() {
            return Err("Vault file not found".into());
        }

        let mut file = File::open(vault_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // Calculate current checksum
        let mut hasher = Sha256::new();
        hasher.update(&buffer);
        let current_hash = hasher.finalize();

        // For now, we'll implement basic integrity check
        // In production, you'd store and compare against a known good hash
        log::info!("Vault integrity check: SHA-256 = {:x}", current_hash);
        
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
        
        std::fs::copy(vault_path, &backup_name)?;
        log::info!("Vault backup created: {}", backup_name);
        
        Ok(backup_name)
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

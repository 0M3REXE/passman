//! Vault Storage Module
//!
//! Provides encrypted on-disk storage for password entries with:
//! - AES-256-GCM authenticated encryption
//! - Argon2id key derivation with configurable parameters
//! - HMAC-SHA256 integrity verification (separate key via HKDF)
//! - Atomic file writes to prevent corruption on crash
//! - Backward-compatible loading of v1/v2 vault formats

use crate::config::get_config;
use crate::crypto::{derive_key, derive_key_pair, decrypt_data, encrypt_data, Argon2Params, Key};
use crate::error::{PassmanError, PassmanResult, VaultError};
use crate::model::Vault;

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::fs::{self, read_dir, File};
use std::io::{Read, Write};
use std::path::Path;
use zeroize::Zeroizing;

type HmacSha256 = Hmac<Sha256>;

/// Default vault file name.
const DEFAULT_VAULT_FILE: &str = "vault.dat";

/// Current vault file format version (v3: separate encryption and HMAC keys).
const VAULT_FORMAT_VERSION: u8 = 3;

/// Magic bytes identifying a passman vault file.
const VAULT_MAGIC: &[u8; 4] = b"PMAN";

// ---------------------------------------------------------------------------
// Vault header
// ---------------------------------------------------------------------------

/// Binary header written at the start of every v2+ vault file.
#[derive(Debug)]
struct VaultHeader {
    magic: [u8; 4],
    version: u8,
    salt_len: u32,
}

impl VaultHeader {
    /// Size of the serialized header in bytes.
    const SIZE: usize = 9; // 4 (magic) + 1 (version) + 4 (salt_len)

    fn new(salt_len: u32) -> Self {
        Self {
            magic: *VAULT_MAGIC,
            version: VAULT_FORMAT_VERSION,
            salt_len,
        }
    }

    fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut buf = [0u8; Self::SIZE];
        buf[0..4].copy_from_slice(&self.magic);
        buf[4] = self.version;
        buf[5..9].copy_from_slice(&self.salt_len.to_le_bytes());
        buf
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < Self::SIZE {
            return None;
        }
        let magic: [u8; 4] = bytes[0..4].try_into().ok()?;
        if &magic != VAULT_MAGIC {
            return None; // Possibly a legacy v1 file
        }
        let version = bytes[4];
        let salt_len = u32::from_le_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]);
        Some(Self {
            magic,
            version,
            salt_len,
        })
    }
}

// ---------------------------------------------------------------------------
// Argon2 parameter helper
// ---------------------------------------------------------------------------

/// Build [`Argon2Params`] from the application configuration.
fn config_argon2_params() -> Argon2Params {
    let cfg = get_config();
    Argon2Params {
        memory_kb: cfg.security.argon2_memory_kb,
        time_cost: cfg.security.argon2_time_cost,
        parallelism: cfg.security.argon2_parallelism,
    }
}

// ---------------------------------------------------------------------------
// VaultManager — public API
// ---------------------------------------------------------------------------

/// Manages encrypted vault files on disk.
///
/// All methods are stateless — they operate directly on the file system and
/// derive keys on-the-fly. Higher-level caching (e.g. keeping the vault in
/// memory) is handled by [`crate::core::PassmanCore`].
pub struct VaultManager;

impl VaultManager {
    // ----- helpers -----

    /// Resolve the vault file path, falling back to the default.
    fn vault_path(vault_file: Option<&str>) -> &str {
        vault_file.unwrap_or(DEFAULT_VAULT_FILE)
    }

    /// HMAC-SHA256 using a dedicated HMAC key.
    fn generate_hmac(hmac_key: &Key, data: &[u8]) -> Vec<u8> {
        let mut mac =
            HmacSha256::new_from_slice(hmac_key.as_ref()).expect("HMAC accepts any key size");
        mac.update(data);
        mac.finalize().into_bytes().to_vec()
    }

    /// Verify HMAC-SHA256 in constant time.
    fn verify_hmac(hmac_key: &Key, data: &[u8], expected: &[u8]) -> bool {
        let mut mac =
            HmacSha256::new_from_slice(hmac_key.as_ref()).expect("HMAC accepts any key size");
        mac.update(data);
        mac.verify_slice(expected).is_ok()
    }

    /// Assemble a complete v3 vault file from its components.
    ///
    /// Layout: `[header(9)][salt][nonce(12)][hmac(32)][ciphertext]`
    ///
    /// The serialized vault plaintext is wrapped in [`Zeroizing`] so it is
    /// automatically wiped from memory after encryption.
    fn assemble_v3_file(
        salt: &SaltString,
        vault: &Vault,
        encryption_key: &Key,
        hmac_key: &Key,
    ) -> PassmanResult<Vec<u8>> {
        let serialized = Zeroizing::new(serde_json::to_vec(vault)?);
        let (ciphertext, nonce) = encrypt_data(encryption_key, &serialized)?;

        let salt_bytes = salt.as_str().as_bytes();
        let header = VaultHeader::new(salt_bytes.len() as u32);

        // HMAC covers nonce || ciphertext
        let mut hmac_data = Vec::with_capacity(nonce.len() + ciphertext.len());
        hmac_data.extend_from_slice(&nonce);
        hmac_data.extend_from_slice(&ciphertext);
        let hmac = Self::generate_hmac(hmac_key, &hmac_data);

        // Pre-allocate the exact output size
        let total = VaultHeader::SIZE + salt_bytes.len() + 12 + 32 + ciphertext.len();
        let mut file_data = Vec::with_capacity(total);
        file_data.extend_from_slice(&header.to_bytes());
        file_data.extend_from_slice(salt_bytes);
        file_data.extend_from_slice(&nonce);
        file_data.extend_from_slice(&hmac);
        file_data.extend_from_slice(&ciphertext);

        Ok(file_data)
    }

    /// Write data atomically: write to `.tmp`, back up existing file to `.bak`, rename.
    fn atomic_write(path: &str, data: &[u8]) -> PassmanResult<()> {
        let temp_path = format!("{}.tmp", path);
        let backup_path = format!("{}.bak", path);

        {
            let mut file = File::create(&temp_path).map_err(|e| {
                PassmanError::Vault(VaultError::WriteError(format!(
                    "failed to create temp file: {}",
                    e
                )))
            })?;
            file.write_all(data)?;
            file.sync_all()?;
        }

        if Path::new(path).exists() {
            let _ = fs::remove_file(&backup_path);
            fs::rename(path, &backup_path)?;
        }

        fs::rename(&temp_path, path)?;
        Ok(())
    }

    // ----- public operations -----

    /// Initialize a new encrypted vault with the given master password.
    pub fn init(
        master_password: &Zeroizing<String>,
        vault_file: Option<&str>,
    ) -> PassmanResult<()> {
        let path = Self::vault_path(vault_file);

        if Path::new(path).exists() {
            return Err(PassmanError::Vault(VaultError::AlreadyExists(
                path.to_string(),
            )));
        }

        let salt = SaltString::generate(&mut OsRng);
        let params = config_argon2_params();
        let key_pair = derive_key_pair(master_password.as_str(), &salt, &params)?;

        let vault = Vault::new();
        let file_data =
            Self::assemble_v3_file(&salt, &vault, &key_pair.encryption_key, &key_pair.hmac_key)?;

        Self::atomic_write(path, &file_data)?;

        log::info!("Vault initialized (v3): {}", path);
        Ok(())
    }

    /// Load and decrypt a vault with the master password.
    ///
    /// Supports v1 (legacy), v2, and v3 formats. v1/v2 vaults emit a warning
    /// suggesting a re-save to upgrade.
    pub fn load(
        master_password: &Zeroizing<String>,
        vault_file: Option<&str>,
    ) -> PassmanResult<Vault> {
        let path = Self::vault_path(vault_file);

        if !Path::new(path).exists() {
            return Err(PassmanError::Vault(VaultError::NotFound(path.to_string())));
        }

        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // Try v2/v3 format first
        if let Some(header) = VaultHeader::from_bytes(&buffer) {
            return Self::load_v2v3(master_password, path, &buffer, &header);
        }

        // Fall back to legacy v1 format
        Self::load_legacy(master_password, path, &buffer)
    }

    /// Load a v2 or v3 format vault.
    fn load_v2v3(
        master_password: &Zeroizing<String>,
        vault_path: &str,
        buffer: &[u8],
        header: &VaultHeader,
    ) -> PassmanResult<Vault> {
        let mut offset = VaultHeader::SIZE;

        // Salt
        let salt_end = offset + header.salt_len as usize;
        if buffer.len() < salt_end + 44 {
            return Err(PassmanError::Vault(VaultError::Corrupted(
                "file too short".into(),
            )));
        }
        let salt_str = std::str::from_utf8(&buffer[offset..salt_end])?;
        let salt = SaltString::from_b64(salt_str)
            .map_err(|e| PassmanError::Vault(VaultError::Corrupted(format!("bad salt: {}", e))))?;
        offset = salt_end;

        // Nonce (12 bytes)
        let nonce: [u8; 12] = buffer[offset..offset + 12].try_into()?;
        offset += 12;

        // Stored HMAC (32 bytes)
        let stored_hmac = &buffer[offset..offset + 32];
        offset += 32;

        // Ciphertext
        let ciphertext = &buffer[offset..];

        // HMAC data = nonce || ciphertext
        let mut hmac_data = Vec::with_capacity(12 + ciphertext.len());
        hmac_data.extend_from_slice(&nonce);
        hmac_data.extend_from_slice(ciphertext);

        let params = config_argon2_params();

        if header.version >= 3 {
            let key_pair = derive_key_pair(master_password.as_str(), &salt, &params)?;

            if !Self::verify_hmac(&key_pair.hmac_key, &hmac_data, stored_hmac) {
                return Err(PassmanError::Vault(VaultError::IntegrityFailed));
            }

            let plaintext = Zeroizing::new(decrypt_data(
                &key_pair.encryption_key,
                ciphertext,
                &nonce,
            )?);
            let vault: Vault = serde_json::from_slice(&plaintext)?;
            log::info!("Vault loaded (v3): {}", vault_path);
            Ok(vault)
        } else {
            // v2 backward compat — single key
            let key = derive_key(master_password.as_str(), &salt)?;

            if !Self::verify_hmac(&key, &hmac_data, stored_hmac) {
                return Err(PassmanError::Vault(VaultError::IntegrityFailed));
            }

            let plaintext = Zeroizing::new(decrypt_data(&key, ciphertext, &nonce)?);
            let vault: Vault = serde_json::from_slice(&plaintext)?;
            log::warn!(
                "Loaded v2 vault: {}. Re-save to upgrade to v3.",
                vault_path
            );
            Ok(vault)
        }
    }

    /// Load a legacy v1 vault (no HMAC, no magic header).
    fn load_legacy(
        master_password: &Zeroizing<String>,
        vault_path: &str,
        buffer: &[u8],
    ) -> PassmanResult<Vault> {
        if buffer.len() < 4 {
            return Err(PassmanError::Vault(VaultError::Corrupted(
                "file too short for v1 header".into(),
            )));
        }

        let mut offset = 0;
        let salt_len = u32::from_le_bytes([
            buffer[offset],
            buffer[offset + 1],
            buffer[offset + 2],
            buffer[offset + 3],
        ]) as usize;
        offset += 4;

        if salt_len > 1000 || buffer.len() < offset + salt_len + 12 {
            return Err(PassmanError::Vault(VaultError::Corrupted(
                "invalid salt length".into(),
            )));
        }

        let salt_str = std::str::from_utf8(&buffer[offset..offset + salt_len])?;
        let salt = SaltString::from_b64(salt_str)
            .map_err(|e| PassmanError::Vault(VaultError::Corrupted(format!("bad salt: {}", e))))?;
        offset += salt_len;

        let nonce: [u8; 12] = buffer[offset..offset + 12].try_into()?;
        offset += 12;

        let ciphertext = &buffer[offset..];
        let key = derive_key(master_password.as_str(), &salt)?;
        let plaintext = Zeroizing::new(decrypt_data(&key, ciphertext, &nonce)?);
        let vault: Vault = serde_json::from_slice(&plaintext)?;

        log::warn!(
            "Loaded legacy v1 vault: {}. Re-save to upgrade.",
            vault_path
        );
        Ok(vault)
    }

    /// Save (re-encrypt) the vault using the v3 format with atomic writes.
    ///
    /// If the vault file already exists the salt is preserved to avoid
    /// unnecessary key derivation parameter changes. A fresh salt is generated
    /// when creating a new file.
    pub fn save(
        vault: &Vault,
        master_password: &Zeroizing<String>,
        vault_file: Option<&str>,
    ) -> PassmanResult<()> {
        let path = Self::vault_path(vault_file);

        // Re-use existing salt when possible to keep the file stable.
        let salt = if Path::new(path).exists() {
            Self::extract_salt(path)?
        } else {
            SaltString::generate(&mut OsRng)
        };

        let params = config_argon2_params();
        let key_pair = derive_key_pair(master_password.as_str(), &salt, &params)?;

        let file_data =
            Self::assemble_v3_file(&salt, vault, &key_pair.encryption_key, &key_pair.hmac_key)?;
        Self::atomic_write(path, &file_data)?;

        log::info!("Vault saved (v3): {}", path);
        Ok(())
    }

    /// Extract the salt from an existing vault file (v1/v2/v3).
    fn extract_salt(path: &str) -> PassmanResult<SaltString> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        if let Some(header) = VaultHeader::from_bytes(&buffer) {
            let salt_end = VaultHeader::SIZE + header.salt_len as usize;
            let salt_str = std::str::from_utf8(&buffer[VaultHeader::SIZE..salt_end])?;
            SaltString::from_b64(salt_str).map_err(|e| {
                PassmanError::Vault(VaultError::Corrupted(format!("bad salt: {}", e)))
            })
        } else {
            // Legacy format
            let salt_len =
                u32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]) as usize;
            let salt_str = std::str::from_utf8(&buffer[4..4 + salt_len])?;
            SaltString::from_b64(salt_str).map_err(|e| {
                PassmanError::Vault(VaultError::Corrupted(format!("bad salt: {}", e)))
            })
        }
    }

    /// Check whether a vault file exists at the given path.
    pub fn exists(vault_file: Option<&str>) -> bool {
        Path::new(Self::vault_path(vault_file)).exists()
    }

    /// List `.dat` vault files in the current directory.
    pub fn list_vaults() -> PassmanResult<Vec<String>> {
        let mut vaults = Vec::new();
        for entry in read_dir(".")? {
            let entry = entry?;
            if let Some(name) = entry.path().file_name().and_then(|n| n.to_str()) {
                if name.ends_with(".dat") {
                    vaults.push(name.to_string());
                }
            }
        }
        vaults.sort();
        Ok(vaults)
    }

    /// Create a timestamped backup, pruning old backups beyond the configured limit.
    pub fn create_backup(vault_file: Option<&str>) -> PassmanResult<String> {
        let path = Self::vault_path(vault_file);

        if !Path::new(path).exists() {
            return Err(PassmanError::Vault(VaultError::NotFound(path.to_string())));
        }

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("{}.bak.{}", path, timestamp);
        fs::copy(path, &backup_name)?;
        log::info!("Vault backup created: {}", backup_name);

        // Prune old backups
        let max_backups = get_config().backup.max_backups;
        let backup_prefix = format!("{}.bak.", path);
        if let Ok(entries) = read_dir(".") {
            let mut backups: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().to_str().map(|s| s.to_string()))
                .filter(|name| name.starts_with(&backup_prefix))
                .collect();
            backups.sort();

            while backups.len() > max_backups {
                if let Some(oldest) = backups.first() {
                    let _ = fs::remove_file(oldest);
                    log::info!("Pruned old backup: {}", oldest);
                    backups.remove(0);
                }
            }
        }

        Ok(backup_name)
    }

    /// Change master password: create backup, load, re-encrypt with new salt & password.
    pub fn change_password(
        old_password: &Zeroizing<String>,
        new_password: &Zeroizing<String>,
        vault_file: Option<&str>,
    ) -> PassmanResult<()> {
        let path = Self::vault_path(vault_file);

        let backup = Self::create_backup(vault_file)?;
        log::info!("Created backup before password change: {}", backup);

        // Load vault with old password (1 key derivation)
        let vault = Self::load(old_password, vault_file)?;

        // Generate fresh salt + key pair for new password (1 key derivation — not 3)
        let new_salt = SaltString::generate(&mut OsRng);
        let params = config_argon2_params();
        let new_key_pair = derive_key_pair(new_password.as_str(), &new_salt, &params)?;

        let file_data = Self::assemble_v3_file(
            &new_salt,
            &vault,
            &new_key_pair.encryption_key,
            &new_key_pair.hmac_key,
        )?;
        Self::atomic_write(path, &file_data)?;

        log::info!("Master password changed successfully (v3): {}", path);
        Ok(())
    }
}

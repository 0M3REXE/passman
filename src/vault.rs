use crate::crypto::{derive_key, encrypt_data, decrypt_data};
use crate::model::Vault;
use argon2::password_hash::SaltString;
use std::fs::{File, read_dir};
use std::io::{Write, Read};
use std::path::Path;

const DEFAULT_VAULT_FILE: &str = "vault.dat";

pub struct VaultManager;

impl VaultManager {
    /// Get the vault file path
    fn get_vault_path(vault_file: Option<&str>) -> &str {
        vault_file.unwrap_or(DEFAULT_VAULT_FILE)
    }

    /// Initialize a new encrypted vault with master password
    pub fn init(master_password: &str, vault_file: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        let vault_path = Self::get_vault_path(vault_file);
        
        if Path::new(vault_path).exists() {
            return Err(format!("Vault '{}' already exists! Remove it to reset.", vault_path).into());
        }        let salt = SaltString::generate(&mut rand::thread_rng());
        let key = derive_key(master_password, &salt)?;

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
    }

    /// Load and decrypt vault with master password
    pub fn load(master_password: &str, vault_file: Option<&str>) -> Result<Vault, Box<dyn std::error::Error>> {
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
        let key = derive_key(master_password, &salt)?;
        let plaintext = decrypt_data(&key, ciphertext, &nonce)?;
        
        let vault: Vault = serde_json::from_slice(&plaintext)?;
        Ok(vault)
    }    /// Save encrypted vault
    pub fn save(vault: &Vault, master_password: &str, vault_file: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
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
        let key = derive_key(master_password, &salt)?;

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
    }
}

use aes_gcm::{Aes256Gcm, KeyInit};
use aes_gcm::aead::{Aead, generic_array::GenericArray};
use argon2::{Argon2, password_hash::SaltString, PasswordHasher};
use sha2::Sha256;
use hmac::{Hmac, Mac as HmacMac};

use zeroize::{Zeroize, ZeroizeOnDrop};

type HmacSha256 = Hmac<Sha256>;

/// Secure key wrapper that automatically zeroizes on drop
#[derive(Clone, ZeroizeOnDrop)]
pub struct Key(GenericArray<u8, typenum::U32>);

impl Key {
    pub fn new(data: GenericArray<u8, typenum::U32>) -> Self {
        Self(data)
    }
}

impl AsRef<[u8]> for Key {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl std::ops::Deref for Key {
    type Target = GenericArray<u8, typenum::U32>;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A pair of keys derived from a master key for domain separation:
/// one for AES-256-GCM encryption, one for HMAC-SHA256 authentication.
pub struct KeyPair {
    pub encryption_key: Key,
    pub hmac_key: Key,
}

#[derive(Debug)]
pub enum CryptoError {
    KeyDerivation(String),
    Encryption(String),
    Decryption(String),
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::KeyDerivation(msg) => write!(f, "Key derivation error: {}", msg),
            CryptoError::Encryption(msg) => write!(f, "Encryption error: {}", msg),
            CryptoError::Decryption(msg) => write!(f, "Decryption error: {}", msg),
        }
    }
}

impl std::error::Error for CryptoError {}

/// Argon2 parameters for key derivation
pub struct Argon2Params {
    pub memory_kb: u32,
    pub time_cost: u32,
    pub parallelism: u32,
}

/// Minimum Argon2 parameter values for security
impl Argon2Params {
    pub const MIN_MEMORY_KB: u32 = 32768;  // 32 MiB
    pub const MIN_TIME_COST: u32 = 2;
    pub const MIN_PARALLELISM: u32 = 1;

    /// Create params with floor validation. Values below minimums are clamped up.
    pub fn new(memory_kb: u32, time_cost: u32, parallelism: u32) -> Self {
        Self {
            memory_kb: memory_kb.max(Self::MIN_MEMORY_KB),
            time_cost: time_cost.max(Self::MIN_TIME_COST),
            parallelism: parallelism.max(Self::MIN_PARALLELISM),
        }
    }

    /// Validate that parameters meet minimum security requirements.
    /// Returns `Err` with a description of which params are too low.
    pub fn validate(&self) -> Result<(), String> {
        let mut issues = Vec::new();
        if self.memory_kb < Self::MIN_MEMORY_KB {
            issues.push(format!("memory_kb={} < minimum {}", self.memory_kb, Self::MIN_MEMORY_KB));
        }
        if self.time_cost < Self::MIN_TIME_COST {
            issues.push(format!("time_cost={} < minimum {}", self.time_cost, Self::MIN_TIME_COST));
        }
        if self.parallelism < Self::MIN_PARALLELISM {
            issues.push(format!("parallelism={} < minimum {}", self.parallelism, Self::MIN_PARALLELISM));
        }
        if issues.is_empty() {
            Ok(())
        } else {
            Err(format!("Argon2 params below security floor: {}", issues.join(", ")))
        }
    }
}

impl Default for Argon2Params {
    fn default() -> Self {
        Self {
            memory_kb: 65536,
            time_cost: 3,
            parallelism: 4,
        }
    }
}

/// Derive a sub-key from a master key using HMAC-SHA256 (HKDF-Expand pattern)
fn derive_subkey(master_key: &[u8], label: &[u8]) -> Key {
    let mut mac = <HmacSha256 as HmacMac>::new_from_slice(master_key)
        .expect("HMAC can take key of any size");
    mac.update(label);
    mac.update(&[0x01]); // counter byte for HKDF-Expand
    let result = mac.finalize().into_bytes();
    Key::new(*GenericArray::from_slice(&result))
}

/// Derive a key pair (encryption + HMAC) from a password and salt.
/// Uses Argon2id to derive a master key, then HKDF-Expand to split
/// into separate encryption and HMAC keys for domain separation.
pub fn derive_key_pair(password: &str, salt: &SaltString, params: &Argon2Params) -> Result<KeyPair, CryptoError> {
    let master_key = derive_key_with_params(password, salt, params)?;
    let encryption_key = derive_subkey(master_key.as_ref(), b"passman-encryption-key");
    let hmac_key = derive_subkey(master_key.as_ref(), b"passman-hmac-key");
    // master_key is dropped here and zeroized via ZeroizeOnDrop
    Ok(KeyPair { encryption_key, hmac_key })
}

/// Derive a single key using Argon2 with configurable parameters.
fn derive_key_with_params(password: &str, salt: &SaltString, params: &Argon2Params) -> Result<Key, CryptoError> {
    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2::Params::new(params.memory_kb, params.time_cost, params.parallelism, None)
            .map_err(|e| CryptoError::KeyDerivation(e.to_string()))?
    );
    
    let hash = argon2.hash_password(password.as_bytes(), salt)
        .map_err(|e| CryptoError::KeyDerivation(e.to_string()))?;
    
    let hash_output = hash.hash
        .ok_or_else(|| CryptoError::KeyDerivation("No hash output".to_string()))?;
    let key_bytes = hash_output.as_bytes();
    
    // Ensure we have exactly 32 bytes for AES-256
    let mut key_array = [0u8; 32];
    let len = std::cmp::min(key_bytes.len(), 32);
    key_array[..len].copy_from_slice(&key_bytes[..len]);
    
    let key = Key::new(*GenericArray::from_slice(&key_array));
    key_array.zeroize();
    
    Ok(key)
}

/// Derive a single key (backward-compatible wrapper using default Argon2 params).
/// Used by legacy code paths and tests.
pub fn derive_key(password: &str, salt: &SaltString) -> Result<Key, CryptoError> {
    derive_key_with_params(password, salt, &Argon2Params::default())
}

pub fn encrypt_data(key: &Key, plaintext: &[u8]) -> Result<(Vec<u8>, [u8; 12]), CryptoError> {
    let cipher = Aes256Gcm::new(key);
    let nonce_bytes = rand::random::<[u8; 12]>();
    let nonce = GenericArray::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, plaintext)
        .map_err(|e| CryptoError::Encryption(e.to_string()))?;
    Ok((ciphertext, nonce_bytes))
}

pub fn decrypt_data(key: &Key, ciphertext: &[u8], nonce: &[u8; 12]) -> Result<Vec<u8>, CryptoError> {
    let cipher = Aes256Gcm::new(key);
    let nonce = GenericArray::from_slice(nonce);
    cipher.decrypt(nonce, ciphertext)
        .map_err(|_e| CryptoError::Decryption("Invalid password or corrupted data".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use argon2::password_hash::rand_core::OsRng;
    
    #[test]
    fn test_key_derivation() {
        let salt = SaltString::generate(&mut OsRng);
        let password = "test_password_123";
        
        let key = derive_key(password, &salt).expect("Key derivation should succeed");
        
        // Key should be 32 bytes for AES-256
        assert_eq!(key.as_ref().len(), 32);
    }
    
    #[test]
    fn test_key_derivation_deterministic() {
        let salt = SaltString::generate(&mut OsRng);
        let password = "deterministic_test";
        
        let key1 = derive_key(password, &salt).expect("Key derivation should succeed");
        let key2 = derive_key(password, &salt).expect("Key derivation should succeed");
        
        // Same password and salt should produce same key
        assert_eq!(key1.as_ref(), key2.as_ref());
    }
    
    #[test]
    fn test_different_passwords_different_keys() {
        let salt = SaltString::generate(&mut OsRng);
        
        let key1 = derive_key("password1", &salt).expect("Key derivation should succeed");
        let key2 = derive_key("password2", &salt).expect("Key derivation should succeed");
        
        // Different passwords should produce different keys
        assert_ne!(key1.as_ref(), key2.as_ref());
    }
    
    #[test]
    fn test_different_salts_different_keys() {
        let salt1 = SaltString::generate(&mut OsRng);
        let salt2 = SaltString::generate(&mut OsRng);
        let password = "same_password";
        
        let key1 = derive_key(password, &salt1).expect("Key derivation should succeed");
        let key2 = derive_key(password, &salt2).expect("Key derivation should succeed");
        
        // Different salts should produce different keys
        assert_ne!(key1.as_ref(), key2.as_ref());
    }
    
    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let salt = SaltString::generate(&mut OsRng);
        let key = derive_key("test_password", &salt).expect("Key derivation should succeed");
        let plaintext = b"Hello, World! This is a secret message.";
        
        let (ciphertext, nonce) = encrypt_data(&key, plaintext).expect("Encryption should succeed");
        let decrypted = decrypt_data(&key, &ciphertext, &nonce).expect("Decryption should succeed");
        
        assert_eq!(decrypted, plaintext);
    }
    
    #[test]
    fn test_ciphertext_different_from_plaintext() {
        let salt = SaltString::generate(&mut OsRng);
        let key = derive_key("test_password", &salt).expect("Key derivation should succeed");
        let plaintext = b"Secret data";
        
        let (ciphertext, _nonce) = encrypt_data(&key, plaintext).expect("Encryption should succeed");
        
        // Ciphertext should be different from plaintext
        assert_ne!(&ciphertext[..], &plaintext[..]);
        
        // Ciphertext should be longer (includes authentication tag)
        assert!(ciphertext.len() > plaintext.len());
    }
    
    #[test]
    fn test_wrong_key_fails_decryption() {
        let salt1 = SaltString::generate(&mut OsRng);
        let salt2 = SaltString::generate(&mut OsRng);
        let key1 = derive_key("password1", &salt1).expect("Key derivation should succeed");
        let key2 = derive_key("password2", &salt2).expect("Key derivation should succeed");
        let plaintext = b"Secret message";
        
        let (ciphertext, nonce) = encrypt_data(&key1, plaintext).expect("Encryption should succeed");
        
        // Decryption with wrong key should fail
        let result = decrypt_data(&key2, &ciphertext, &nonce);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_wrong_nonce_fails_decryption() {
        let salt = SaltString::generate(&mut OsRng);
        let key = derive_key("test_password", &salt).expect("Key derivation should succeed");
        let plaintext = b"Secret message";
        
        let (ciphertext, _nonce) = encrypt_data(&key, plaintext).expect("Encryption should succeed");
        let wrong_nonce = rand::random::<[u8; 12]>();
        
        // Decryption with wrong nonce should fail
        let result = decrypt_data(&key, &ciphertext, &wrong_nonce);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_tampered_ciphertext_fails() {
        let salt = SaltString::generate(&mut OsRng);
        let key = derive_key("test_password", &salt).expect("Key derivation should succeed");
        let plaintext = b"Secret message";
        
        let (mut ciphertext, nonce) = encrypt_data(&key, plaintext).expect("Encryption should succeed");
        
        // Tamper with ciphertext
        if !ciphertext.is_empty() {
            ciphertext[0] ^= 0xFF;
        }
        
        // Decryption of tampered ciphertext should fail
        let result = decrypt_data(&key, &ciphertext, &nonce);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_empty_plaintext() {
        let salt = SaltString::generate(&mut OsRng);
        let key = derive_key("test_password", &salt).expect("Key derivation should succeed");
        let plaintext = b"";
        
        let (ciphertext, nonce) = encrypt_data(&key, plaintext).expect("Encryption should succeed");
        let decrypted = decrypt_data(&key, &ciphertext, &nonce).expect("Decryption should succeed");
        
        assert_eq!(decrypted, plaintext);
    }
    
    #[test]
    fn test_large_plaintext() {
        let salt = SaltString::generate(&mut OsRng);
        let key = derive_key("test_password", &salt).expect("Key derivation should succeed");
        let plaintext: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
        
        let (ciphertext, nonce) = encrypt_data(&key, &plaintext).expect("Encryption should succeed");
        let decrypted = decrypt_data(&key, &ciphertext, &nonce).expect("Decryption should succeed");
        
        assert_eq!(decrypted, plaintext);
    }
    
    #[test]
    fn test_crypto_error_display() {
        let err = CryptoError::KeyDerivation("test error".to_string());
        assert!(err.to_string().contains("Key derivation"));
        
        let err = CryptoError::Encryption("enc error".to_string());
        assert!(err.to_string().contains("Encryption"));
        
        let err = CryptoError::Decryption("dec error".to_string());
        assert!(err.to_string().contains("Decryption"));
    }
}
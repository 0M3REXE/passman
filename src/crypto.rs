use aes_gcm::{Aes256Gcm, KeyInit};
use aes_gcm::aead::{Aead, generic_array::GenericArray};
use argon2::{Argon2, password_hash::SaltString, PasswordHasher};
use rand;

pub type Key = GenericArray<u8, typenum::U32>;

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

pub fn derive_key(password: &str, salt: &SaltString) -> Result<Key, CryptoError> {
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), salt)
        .map_err(|e| CryptoError::KeyDerivation(e.to_string()))?;
    let hash_output = hash.hash
        .ok_or_else(|| CryptoError::KeyDerivation("No hash output".to_string()))?;
    let key_bytes = hash_output.as_bytes();
    
    // Ensure we have exactly 32 bytes for AES-256
    let mut key_array = [0u8; 32];
    let len = std::cmp::min(key_bytes.len(), 32);
    key_array[..len].copy_from_slice(&key_bytes[..len]);
    
    Ok(*GenericArray::from_slice(&key_array))
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
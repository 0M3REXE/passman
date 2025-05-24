use aes_gcm::{Aes256Gcm, KeyInit};
use aes_gcm::aead::{Aead, generic_array::GenericArray};
use argon2::{Argon2, password_hash::SaltString, PasswordHasher};
use rand;

pub type Key = GenericArray<u8, typenum::U32>;

pub fn derive_key(password: &str, salt: &SaltString) -> Key {
    let argon2 = Argon2::default();
    let hash = argon2.hash_password(password.as_bytes(), salt).unwrap();
    let hash_output = hash.hash.unwrap();
    let key_bytes = hash_output.as_bytes();
    
    // Ensure we have exactly 32 bytes for AES-256
    let mut key_array = [0u8; 32];
    let len = std::cmp::min(key_bytes.len(), 32);
    key_array[..len].copy_from_slice(&key_bytes[..len]);
    
    *GenericArray::from_slice(&key_array)
}

pub fn encrypt_data(key: &Key, plaintext: &[u8]) -> (Vec<u8>, [u8; 12]) {
    let cipher = Aes256Gcm::new(key);
    let nonce_bytes = rand::random::<[u8; 12]>();
    let nonce = GenericArray::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, plaintext).unwrap();
    (ciphertext, nonce_bytes)
}

pub fn decrypt_data(key: &Key, ciphertext: &[u8], nonce: &[u8; 12]) -> Vec<u8> {
    let cipher = Aes256Gcm::new(key);
    let nonce = GenericArray::from_slice(nonce);
    cipher.decrypt(nonce, ciphertext).unwrap()
}
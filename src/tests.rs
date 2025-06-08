#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_vault_creation_and_loading() {
        let temp_dir = tempdir().unwrap();
        let vault_path = temp_dir.path().join("test_vault.dat");
        let vault_path_str = vault_path.to_str().unwrap();
        
        let password = zeroize::Zeroizing::new("test_password123".to_string());
        
        // Create vault
        VaultManager::init(&password, Some(vault_path_str)).unwrap();
        assert!(vault_path.exists());
        
        // Load vault
        let vault = VaultManager::load(&password, Some(vault_path_str)).unwrap();
        assert!(vault.is_empty());
    }
    
    #[test]
    fn test_entry_operations() {
        let temp_dir = tempdir().unwrap();
        let vault_path = temp_dir.path().join("test_vault.dat");
        let vault_path_str = vault_path.to_str().unwrap();
        
        let password = zeroize::Zeroizing::new("test_password123".to_string());
        
        // Initialize vault
        VaultManager::init(&password, Some(vault_path_str)).unwrap();
        let mut vault = VaultManager::load(&password, Some(vault_path_str)).unwrap();
        
        // Add entry
        let entry = Entry::new(
            "test_user".to_string(),
            "test_pass".to_string(),
            Some("test note".to_string())
        );
        vault.add_entry("test_id".to_string(), entry);
        
        // Save and reload
        VaultManager::save(&vault, &password, Some(vault_path_str)).unwrap();
        let reloaded_vault = VaultManager::load(&password, Some(vault_path_str)).unwrap();
        
        // Verify entry exists
        let retrieved_entry = reloaded_vault.get_entry("test_id").unwrap();
        assert_eq!(retrieved_entry.username, "test_user");
        assert_eq!(retrieved_entry.password, "test_pass");
        assert_eq!(retrieved_entry.note, Some("test note".to_string()));
    }
    
    #[test]
    fn test_wrong_password() {
        let temp_dir = tempdir().unwrap();
        let vault_path = temp_dir.path().join("test_vault.dat");
        let vault_path_str = vault_path.to_str().unwrap();
        
        let correct_password = zeroize::Zeroizing::new("correct_password".to_string());
        let wrong_password = zeroize::Zeroizing::new("wrong_password".to_string());
        
        // Create vault with correct password
        VaultManager::init(&correct_password, Some(vault_path_str)).unwrap();
        
        // Try to load with wrong password
        let result = VaultManager::load(&wrong_password, Some(vault_path_str));
        assert!(result.is_err());
    }
    
    #[test]
    fn test_password_strength_analysis() {
        let (strength, suggestions) = analyze_password_strength("weak");
        assert_eq!(strength, PasswordStrength::VeryWeak);
        assert!(!suggestions.is_empty());
        
        let (strength, suggestions) = analyze_password_strength("VeryStr0ng!Password#123");
        assert_eq!(strength, PasswordStrength::Strong);
        assert!(suggestions.is_empty());
    }
    
    #[test]
    fn test_password_generation() {
        let password = generate_password(16);
        assert_eq!(password.len(), 16);
        
        // Test that generated passwords are different
        let password2 = generate_password(16);
        assert_ne!(password, password2);
    }
}

#[cfg(test)]
mod crypto_tests {
    use super::crypto::*;
    use argon2::password_hash::SaltString;
    use rand::thread_rng;
    
    #[test]
    fn test_encryption_decryption() {
        let salt = SaltString::generate(&mut thread_rng());
        let password = "test_password";
        let key = derive_key(password, &salt).unwrap();
        
        let plaintext = b"Hello, World!";
        let (ciphertext, nonce) = encrypt_data(&key, plaintext).unwrap();
        let decrypted = decrypt_data(&key, &ciphertext, &nonce).unwrap();
        
        assert_eq!(plaintext, decrypted.as_slice());
    }
    
    #[test]
    fn test_key_derivation_consistency() {
        let salt = SaltString::generate(&mut thread_rng());
        let password = "test_password";
        
        let key1 = derive_key(password, &salt).unwrap();
        let key2 = derive_key(password, &salt).unwrap();
        
        assert_eq!(key1.as_ref(), key2.as_ref());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_full_workflow() {
        let temp_dir = tempdir().unwrap();
        let vault_path = temp_dir.path().join("integration_test.dat");
        let vault_path_str = vault_path.to_str().unwrap();
        
        let password = zeroize::Zeroizing::new("integration_test_password".to_string());
        
        // 1. Initialize vault
        VaultManager::init(&password, Some(vault_path_str)).unwrap();
        
        // 2. Add multiple entries
        let mut vault = VaultManager::load(&password, Some(vault_path_str)).unwrap();
        
        for i in 0..5 {
            let entry = Entry::new(
                format!("user_{}", i),
                format!("password_{}", i),
                Some(format!("note_{}", i))
            );
            vault.add_entry(format!("entry_{}", i), entry);
        }
        
        VaultManager::save(&vault, &password, Some(vault_path_str)).unwrap();
        
        // 3. Reload and verify all entries
        let reloaded_vault = VaultManager::load(&password, Some(vault_path_str)).unwrap();
        assert_eq!(reloaded_vault.entries.len(), 5);
        
        for i in 0..5 {
            let entry = reloaded_vault.get_entry(&format!("entry_{}", i)).unwrap();
            assert_eq!(entry.username, format!("user_{}", i));
            assert_eq!(entry.password, format!("password_{}", i));
            assert_eq!(entry.note, Some(format!("note_{}", i)));
        }
        
        // 4. Remove an entry
        let mut vault = reloaded_vault;
        vault.remove_entry("entry_2");
        VaultManager::save(&vault, &password, Some(vault_path_str)).unwrap();
        
        // 5. Verify removal
        let final_vault = VaultManager::load(&password, Some(vault_path_str)).unwrap();
        assert_eq!(final_vault.entries.len(), 4);
        assert!(final_vault.get_entry("entry_2").is_none());
    }
}

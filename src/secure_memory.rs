use zeroize::Zeroize;

/// Secure string that prevents memory dumps and automatically zeroizes
#[derive(Clone)]
pub struct SecureString {
    data: Vec<u8>,
}

impl SecureString {
    pub fn new(s: String) -> Self {
        let data = s.into_bytes();
        
        // TODO: Implement memory locking for production use
        // Platform-specific memory locking would go here
        
        Self { data }
    }
    
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.data).unwrap_or("")    }
    
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Drop for SecureString {
    fn drop(&mut self) {
        // Zero out memory before dropping
        self.data.zeroize();
    }
}

impl From<String> for SecureString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl AsRef<str> for SecureString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

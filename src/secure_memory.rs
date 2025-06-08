use zeroize::Zeroize;

#[cfg(windows)]
use windows::Win32::System::Memory::{VirtualLock, VirtualUnlock};

#[cfg(unix)]
use libc::{mlock, munlock};

/// Secure string that prevents memory dumps and automatically zeroizes
#[derive(Clone)]
pub struct SecureString {
    data: Vec<u8>,
    #[cfg(any(windows, unix))]
    locked: bool,
}

impl SecureString {
    pub fn new(s: String) -> Self {
        let mut data = s.into_bytes();
        
        #[cfg(any(windows, unix))]
        let locked = Self::lock_memory(&mut data);
        
        #[cfg(not(any(windows, unix)))]
        let _locked = false;
        
        Self { 
            data,
            #[cfg(any(windows, unix))]
            locked,
        }
    }
    
    #[cfg(windows)]
    fn lock_memory(data: &mut [u8]) -> bool {
        if data.is_empty() {
            return false;
        }
        
        unsafe {
            VirtualLock(
                data.as_mut_ptr() as *mut std::ffi::c_void,
                data.len()
            ).is_ok()
        }
    }
    
    #[cfg(unix)]
    fn lock_memory(data: &mut [u8]) -> bool {
        if data.is_empty() {
            return false;
        }
        
        unsafe {
            mlock(
                data.as_ptr() as *const std::ffi::c_void,
                data.len()
            ) == 0
        }
    }
    
    #[cfg(not(any(windows, unix)))]
    fn lock_memory(_data: &mut [u8]) -> bool {
        // Memory locking not implemented for this platform
        log::warn!("Memory locking not available on this platform");
        false
    }
    
    #[cfg(windows)]
    fn unlock_memory(&mut self) {
        if self.locked && !self.data.is_empty() {
            unsafe {
                let _ = VirtualUnlock(
                    self.data.as_mut_ptr() as *mut std::ffi::c_void,
                    self.data.len()
                );
            }
        }
    }
    
    #[cfg(unix)]
    fn unlock_memory(&mut self) {
        if self.locked && !self.data.is_empty() {
            unsafe {
                let _ = munlock(
                    self.data.as_ptr() as *const std::ffi::c_void,
                    self.data.len()
                );
            }
        }
    }
    
    #[cfg(not(any(windows, unix)))]
    fn unlock_memory(&mut self) {
        // Nothing to do on unsupported platforms
    }
    
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.data).unwrap_or("")
    }
    
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
        // Unlock memory before zeroizing and dropping
        #[cfg(any(windows, unix))]
        self.unlock_memory();
        
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

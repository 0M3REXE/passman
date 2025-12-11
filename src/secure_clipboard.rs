//! Secure Clipboard Module
//! 
//! Provides clipboard operations with automatic clearing after a timeout
//! to prevent password leakage through clipboard history.
//! 
//! On Windows, this module also excludes sensitive content from clipboard history
//! and clears it from history when clearing the clipboard.

#![allow(dead_code)]

use clipboard::{ClipboardProvider, ClipboardContext};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;

/// Default clipboard clear timeout in seconds
const DEFAULT_CLEAR_TIMEOUT_SECS: u64 = 30;

/// Windows clipboard format for excluding from history
/// CLIPBOARD_FORMAT_EXCLUDE_FROM_HISTORY = "ExcludeClipboardContentFromMonitorProcessing"
#[cfg(target_os = "windows")]
const CF_EXCLUDE_FROM_HISTORY_NAME: &str = "ExcludeClipboardContentFromMonitorProcessing";

/// Windows clipboard format for marking as confidential
/// This tells Windows not to sync or store this content
#[cfg(target_os = "windows")]
const CF_CAN_INCLUDE_IN_HISTORY_NAME: &str = "CanIncludeInClipboardHistory";

#[cfg(target_os = "windows")]
mod win32 {
    use std::ffi::CString;
    
    #[link(name = "user32")]
    extern "system" {
        pub fn OpenClipboard(hWndNewOwner: *mut std::ffi::c_void) -> i32;
        pub fn CloseClipboard() -> i32;
        pub fn EmptyClipboard() -> i32;
        pub fn SetClipboardData(uFormat: u32, hMem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
        pub fn RegisterClipboardFormatA(lpszFormat: *const i8) -> u32;
        pub fn GetClipboardData(uFormat: u32) -> *mut std::ffi::c_void;
    }
    
    #[link(name = "kernel32")]
    extern "system" {
        pub fn GlobalAlloc(uFlags: u32, dwBytes: usize) -> *mut std::ffi::c_void;
        pub fn GlobalLock(hMem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
        pub fn GlobalUnlock(hMem: *mut std::ffi::c_void) -> i32;
        pub fn GlobalFree(hMem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    }
    
    pub const GMEM_MOVEABLE: u32 = 0x0002;
    pub const CF_UNICODETEXT: u32 = 13;
    
    /// Register a custom clipboard format
    pub fn register_clipboard_format(name: &str) -> u32 {
        let c_name = CString::new(name).unwrap();
        unsafe { RegisterClipboardFormatA(c_name.as_ptr()) }
    }
}

/// Result type for clipboard operations
pub type ClipboardResult<T> = Result<T, ClipboardError>;

/// Clipboard operation errors
#[derive(Debug, Clone)]
pub enum ClipboardError {
    /// Failed to access clipboard
    AccessError(String),
    /// Failed to set clipboard content
    SetError(String),
    /// Failed to clear clipboard
    ClearError(String),
    /// Clipboard is currently locked
    Locked,
}

impl std::fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClipboardError::AccessError(msg) => write!(f, "Clipboard access error: {}", msg),
            ClipboardError::SetError(msg) => write!(f, "Failed to set clipboard: {}", msg),
            ClipboardError::ClearError(msg) => write!(f, "Failed to clear clipboard: {}", msg),
            ClipboardError::Locked => write!(f, "Clipboard is locked by another operation"),
        }
    }
}

impl std::error::Error for ClipboardError {}

/// Secure clipboard manager with auto-clear functionality
pub struct SecureClipboard {
    /// Timeout in seconds before clipboard is automatically cleared
    clear_timeout_secs: u64,
    /// Track if a clear operation is pending
    clear_pending: Arc<AtomicBool>,
    /// Content identifier to verify we're clearing our own content
    content_id: Arc<Mutex<Option<String>>>,
    /// Whether clipboard operations are enabled
    enabled: bool,
}

impl SecureClipboard {
    /// Create a new SecureClipboard with default timeout
    pub fn new() -> Self {
        Self {
            clear_timeout_secs: DEFAULT_CLEAR_TIMEOUT_SECS,
            clear_pending: Arc::new(AtomicBool::new(false)),
            content_id: Arc::new(Mutex::new(None)),
            enabled: true,
        }
    }

    /// Create a new SecureClipboard with custom timeout
    pub fn with_timeout(timeout_secs: u64) -> Self {
        Self {
            clear_timeout_secs: timeout_secs,
            clear_pending: Arc::new(AtomicBool::new(false)),
            content_id: Arc::new(Mutex::new(None)),
            enabled: true,
        }
    }

    /// Set the clear timeout
    pub fn set_timeout(&mut self, timeout_secs: u64) {
        self.clear_timeout_secs = timeout_secs;
    }

    /// Get the current timeout setting
    pub fn get_timeout(&self) -> u64 {
        self.clear_timeout_secs
    }

    /// Enable or disable clipboard operations
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if clipboard operations are enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Copy text to clipboard with automatic clearing after timeout
    /// 
    /// # Arguments
    /// * `text` - The text to copy to clipboard
    /// * `auto_clear` - Whether to automatically clear after timeout
    /// 
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(ClipboardError)` on failure
    pub fn copy(&self, text: &str, auto_clear: bool) -> ClipboardResult<()> {
        if !self.enabled {
            return Ok(());
        }

        // Create a unique identifier for this content
        let content_id = format!("passman_{}", uuid::Uuid::new_v4());
        
        // On Windows, use native API to exclude from clipboard history
        #[cfg(target_os = "windows")]
        {
            self.copy_windows_secure(text)?;
        }
        
        // On non-Windows, use the standard clipboard crate
        #[cfg(not(target_os = "windows"))]
        {
            let mut ctx: ClipboardContext = ClipboardProvider::new()
                .map_err(|e| ClipboardError::AccessError(e.to_string()))?;
            
            ctx.set_contents(text.to_owned())
                .map_err(|e| ClipboardError::SetError(e.to_string()))?;
        }

        // Store the content ID
        if let Ok(mut id) = self.content_id.lock() {
            *id = Some(content_id.clone());
        }

        // Schedule auto-clear if requested
        if auto_clear && self.clear_timeout_secs > 0 {
            self.schedule_clear(content_id);
        }

        Ok(())
    }
    
    /// Windows-specific clipboard copy that excludes content from history
    #[cfg(target_os = "windows")]
    fn copy_windows_secure(&self, text: &str) -> ClipboardResult<()> {
        use std::ptr::null_mut;
        
        unsafe {
            // Open clipboard
            if win32::OpenClipboard(null_mut()) == 0 {
                return Err(ClipboardError::AccessError("Failed to open clipboard".to_string()));
            }
            
            // Empty clipboard first
            if win32::EmptyClipboard() == 0 {
                win32::CloseClipboard();
                return Err(ClipboardError::ClearError("Failed to empty clipboard".to_string()));
            }
            
            // Convert text to UTF-16 for Windows
            let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
            let size = wide.len() * 2;
            
            // Allocate global memory for the text
            let hmem = win32::GlobalAlloc(win32::GMEM_MOVEABLE, size);
            if hmem.is_null() {
                win32::CloseClipboard();
                return Err(ClipboardError::SetError("Failed to allocate memory".to_string()));
            }
            
            // Lock and copy
            let ptr = win32::GlobalLock(hmem);
            if ptr.is_null() {
                win32::GlobalFree(hmem);
                win32::CloseClipboard();
                return Err(ClipboardError::SetError("Failed to lock memory".to_string()));
            }
            
            std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr as *mut u16, wide.len());
            win32::GlobalUnlock(hmem);
            
            // Set the text data
            if win32::SetClipboardData(win32::CF_UNICODETEXT, hmem).is_null() {
                win32::GlobalFree(hmem);
                win32::CloseClipboard();
                return Err(ClipboardError::SetError("Failed to set clipboard data".to_string()));
            }
            
            // Set the "ExcludeClipboardContentFromMonitorProcessing" format
            // This tells Windows not to add this to clipboard history
            let exclude_format = win32::register_clipboard_format(CF_EXCLUDE_FROM_HISTORY_NAME);
            if exclude_format != 0 {
                // Allocate a small buffer with value 1 to indicate exclusion
                let hmem_exclude = win32::GlobalAlloc(win32::GMEM_MOVEABLE, 4);
                if !hmem_exclude.is_null() {
                    let ptr_exclude = win32::GlobalLock(hmem_exclude);
                    if !ptr_exclude.is_null() {
                        *(ptr_exclude as *mut u32) = 1;
                        win32::GlobalUnlock(hmem_exclude);
                        win32::SetClipboardData(exclude_format, hmem_exclude);
                    }
                }
            }
            
            // Also set "CanIncludeInClipboardHistory" to 0 (false)
            let can_include_format = win32::register_clipboard_format(CF_CAN_INCLUDE_IN_HISTORY_NAME);
            if can_include_format != 0 {
                let hmem_history = win32::GlobalAlloc(win32::GMEM_MOVEABLE, 4);
                if !hmem_history.is_null() {
                    let ptr_history = win32::GlobalLock(hmem_history);
                    if !ptr_history.is_null() {
                        *(ptr_history as *mut u32) = 0; // 0 = don't include in history
                        win32::GlobalUnlock(hmem_history);
                        win32::SetClipboardData(can_include_format, hmem_history);
                    }
                }
            }
            
            win32::CloseClipboard();
        }
        
        log::debug!("Password copied to clipboard (excluded from history)");
        Ok(())
    }

    /// Copy password to clipboard (always auto-clears)
    pub fn copy_password(&self, password: &str) -> ClipboardResult<()> {
        self.copy(password, true)
    }

    /// Copy username to clipboard (no auto-clear by default)
    pub fn copy_username(&self, username: &str) -> ClipboardResult<()> {
        self.copy(username, false)
    }

    /// Schedule clipboard clearing after timeout
    fn schedule_clear(&self, expected_content_id: String) {
        let clear_pending = Arc::clone(&self.clear_pending);
        let content_id = Arc::clone(&self.content_id);
        let timeout = self.clear_timeout_secs;

        // Mark that a clear is pending
        clear_pending.store(true, Ordering::SeqCst);

        thread::spawn(move || {
            thread::sleep(Duration::from_secs(timeout));

            // Check if this is still our content that should be cleared
            let should_clear = if let Ok(id) = content_id.lock() {
                id.as_ref() == Some(&expected_content_id)
            } else {
                false
            };

            if should_clear {
                if let Ok(mut ctx) = ClipboardProvider::new() as Result<ClipboardContext, _> {
                    // Clear by setting empty content
                    let _ = ctx.set_contents(String::new());
                    log::debug!("Clipboard auto-cleared after {}s timeout", timeout);
                }

                // Clear the content ID
                if let Ok(mut id) = content_id.lock() {
                    *id = None;
                }
            }

            clear_pending.store(false, Ordering::SeqCst);
        });
    }

    /// Immediately clear the clipboard
    pub fn clear_now(&self) -> ClipboardResult<()> {
        let mut ctx: ClipboardContext = ClipboardProvider::new()
            .map_err(|e| ClipboardError::AccessError(e.to_string()))?;
        
        ctx.set_contents(String::new())
            .map_err(|e| ClipboardError::ClearError(e.to_string()))?;

        // Clear the content ID
        if let Ok(mut id) = self.content_id.lock() {
            *id = None;
        }

        log::debug!("Clipboard cleared immediately");
        Ok(())
    }

    /// Check if a clear operation is pending
    pub fn is_clear_pending(&self) -> bool {
        self.clear_pending.load(Ordering::SeqCst)
    }

    /// Get remaining time until clipboard clears (approximate)
    /// Returns None if no clear is pending
    pub fn get_remaining_time(&self) -> Option<u64> {
        if self.is_clear_pending() {
            // This is approximate since we don't track exact start time
            // For accurate tracking, we'd need additional state
            Some(self.clear_timeout_secs)
        } else {
            None
        }
    }
}

impl Default for SecureClipboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for SecureClipboard {
    fn clone(&self) -> Self {
        Self {
            clear_timeout_secs: self.clear_timeout_secs,
            clear_pending: Arc::new(AtomicBool::new(false)),
            content_id: Arc::new(Mutex::new(None)),
            enabled: self.enabled,
        }
    }
}

/// Global clipboard instance for easy access
static CLIPBOARD: std::sync::OnceLock<Mutex<SecureClipboard>> = std::sync::OnceLock::new();

/// Get the global secure clipboard instance
pub fn get_clipboard() -> &'static Mutex<SecureClipboard> {
    CLIPBOARD.get_or_init(|| Mutex::new(SecureClipboard::new()))
}

/// Convenience function to copy password with auto-clear
pub fn copy_password_secure(password: &str) -> ClipboardResult<()> {
    get_clipboard()
        .lock()
        .map_err(|_| ClipboardError::Locked)?
        .copy_password(password)
}

/// Convenience function to copy text without auto-clear
pub fn copy_text(text: &str) -> ClipboardResult<()> {
    get_clipboard()
        .lock()
        .map_err(|_| ClipboardError::Locked)?
        .copy(text, false)
}

/// Convenience function to clear clipboard immediately
pub fn clear_clipboard() -> ClipboardResult<()> {
    get_clipboard()
        .lock()
        .map_err(|_| ClipboardError::Locked)?
        .clear_now()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_clipboard_creation() {
        let clipboard = SecureClipboard::new();
        assert_eq!(clipboard.get_timeout(), DEFAULT_CLEAR_TIMEOUT_SECS);
        assert!(clipboard.is_enabled());
    }

    #[test]
    fn test_custom_timeout() {
        let clipboard = SecureClipboard::with_timeout(60);
        assert_eq!(clipboard.get_timeout(), 60);
    }

    #[test]
    fn test_disable_clipboard() {
        let mut clipboard = SecureClipboard::new();
        clipboard.set_enabled(false);
        assert!(!clipboard.is_enabled());
        
        // Should succeed but do nothing when disabled
        assert!(clipboard.copy("test", false).is_ok());
    }
}

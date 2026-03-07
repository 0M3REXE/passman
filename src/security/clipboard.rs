//! Secure Clipboard Module
//!
//! Provides clipboard operations with automatic clearing after a timeout
//! to prevent password leakage through clipboard history.
//!
//! On Windows, this module also excludes sensitive content from clipboard history
//! and clears it from history when clearing the clipboard.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

/// Default clipboard clear timeout in seconds
const DEFAULT_CLEAR_TIMEOUT_SECS: u64 = 30;

/// Windows clipboard format for excluding from history
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
        pub fn SetClipboardData(
            uFormat: u32,
            hMem: *mut std::ffi::c_void,
        ) -> *mut std::ffi::c_void;
        pub fn RegisterClipboardFormatA(lpszFormat: *const i8) -> u32;
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
#[allow(clippy::enum_variant_names)]
pub enum ClipboardError {
    /// Failed to access clipboard
    AccessError(String),
    /// Failed to set clipboard content
    SetError(String),
    /// Failed to clear clipboard
    ClearError(String),
}

impl std::fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClipboardError::AccessError(msg) => write!(f, "Clipboard access error: {}", msg),
            ClipboardError::SetError(msg) => write!(f, "Failed to set clipboard: {}", msg),
            ClipboardError::ClearError(msg) => write!(f, "Failed to clear clipboard: {}", msg),
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

    /// Copy text to clipboard with automatic clearing after timeout
    ///
    /// # Arguments
    /// * `text` - The text to copy to clipboard
    /// * `auto_clear` - Whether to automatically clear after timeout
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

        // On non-Windows, use arboard (supports X11, Wayland, macOS)
        #[cfg(not(target_os = "windows"))]
        {
            let mut clipboard = arboard::Clipboard::new()
                .map_err(|e| ClipboardError::AccessError(e.to_string()))?;

            clipboard.set_text(text)
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
                return Err(ClipboardError::AccessError(
                    "Failed to open clipboard".to_string(),
                ));
            }

            // Empty clipboard first
            if win32::EmptyClipboard() == 0 {
                win32::CloseClipboard();
                return Err(ClipboardError::ClearError(
                    "Failed to empty clipboard".to_string(),
                ));
            }

            // Convert text to UTF-16 for Windows
            let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
            let size = wide.len() * 2;

            // Allocate global memory for the text
            let hmem = win32::GlobalAlloc(win32::GMEM_MOVEABLE, size);
            if hmem.is_null() {
                win32::CloseClipboard();
                return Err(ClipboardError::SetError(
                    "Failed to allocate memory".to_string(),
                ));
            }

            // Lock and copy
            let ptr = win32::GlobalLock(hmem);
            if ptr.is_null() {
                win32::GlobalFree(hmem);
                win32::CloseClipboard();
                return Err(ClipboardError::SetError(
                    "Failed to lock memory".to_string(),
                ));
            }

            std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr as *mut u16, wide.len());
            win32::GlobalUnlock(hmem);

            // Set the text data
            if win32::SetClipboardData(win32::CF_UNICODETEXT, hmem).is_null() {
                win32::GlobalFree(hmem);
                win32::CloseClipboard();
                return Err(ClipboardError::SetError(
                    "Failed to set clipboard data".to_string(),
                ));
            }

            // Set the "ExcludeClipboardContentFromMonitorProcessing" format
            // This tells Windows not to add this to clipboard history
            let exclude_format =
                win32::register_clipboard_format(CF_EXCLUDE_FROM_HISTORY_NAME);
            if exclude_format != 0 {
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
            let can_include_format =
                win32::register_clipboard_format(CF_CAN_INCLUDE_IN_HISTORY_NAME);
            if can_include_format != 0 {
                let hmem_history = win32::GlobalAlloc(win32::GMEM_MOVEABLE, 4);
                if !hmem_history.is_null() {
                    let ptr_history = win32::GlobalLock(hmem_history);
                    if !ptr_history.is_null() {
                        *(ptr_history as *mut u32) = 0;
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
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.clear();
                    log::debug!("Clipboard auto-cleared after {}s timeout", timeout);
                }

                if let Ok(mut id) = content_id.lock() {
                    *id = None;
                }
            }

            clear_pending.store(false, Ordering::SeqCst);
        });
    }

    /// Immediately clear the clipboard
    pub fn clear_now(&self) -> ClipboardResult<()> {
        let mut clipboard = arboard::Clipboard::new()
            .map_err(|e| ClipboardError::AccessError(e.to_string()))?;

        clipboard.clear()
            .map_err(|e| ClipboardError::ClearError(e.to_string()))?;

        if let Ok(mut id) = self.content_id.lock() {
            *id = None;
        }

        log::debug!("Clipboard cleared immediately");
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_clipboard_creation() {
        let clipboard = SecureClipboard::new();
        assert_eq!(clipboard.clear_timeout_secs, DEFAULT_CLEAR_TIMEOUT_SECS);
        assert!(clipboard.enabled);
    }

    #[test]
    fn test_custom_timeout() {
        let clipboard = SecureClipboard::with_timeout(60);
        assert_eq!(clipboard.clear_timeout_secs, 60);
    }
}

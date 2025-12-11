//! Session Manager Module
//! 
//! Handles session state, auto-lock timeout, and security policies
//! for the password manager application.

#![allow(dead_code)]

use std::time::{Duration, Instant};
#[allow(unused_imports)]
use zeroize::Zeroizing;
use crate::crypto::Key;

/// Default auto-lock timeout in seconds (5 minutes)
const DEFAULT_LOCK_TIMEOUT_SECS: u64 = 300;

/// Maximum failed login attempts before lockout
const MAX_FAILED_ATTEMPTS: u32 = 5;

/// Base lockout duration in seconds
const BASE_LOCKOUT_SECS: u64 = 30;

/// Session state enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum SessionState {
    /// No vault is loaded
    Locked,
    /// Vault is unlocked and active
    Unlocked,
    /// Session timed out, requires re-authentication
    TimedOut,
    /// Account is locked due to failed attempts
    LockedOut { remaining_secs: u64 },
}

/// Configuration for session behavior
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Auto-lock timeout in seconds (0 = disabled)
    pub lock_timeout_secs: u64,
    /// Whether to clear clipboard on lock
    pub clear_clipboard_on_lock: bool,
    /// Whether to lock on minimize
    pub lock_on_minimize: bool,
    /// Whether to lock on screen lock
    pub lock_on_screen_lock: bool,
    /// Maximum failed login attempts
    pub max_failed_attempts: u32,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            lock_timeout_secs: DEFAULT_LOCK_TIMEOUT_SECS,
            clear_clipboard_on_lock: true,
            lock_on_minimize: false,
            lock_on_screen_lock: true,
            max_failed_attempts: MAX_FAILED_ATTEMPTS,
        }
    }
}

/// Session manager for handling authentication state and timeouts
pub struct SessionManager {
    /// Current session state
    state: SessionState,
    /// Last activity timestamp
    last_activity: Option<Instant>,
    /// Session configuration
    config: SessionConfig,
    /// Failed login attempts counter
    failed_attempts: u32,
    /// Lockout start time
    lockout_start: Option<Instant>,
    /// Current vault file path
    vault_file: Option<String>,
    /// Derived encryption key (kept in memory while unlocked)
    encryption_key: Option<Key>,
    /// Session start time
    session_start: Option<Instant>,
}

impl SessionManager {
    /// Create a new session manager with default configuration
    pub fn new() -> Self {
        Self {
            state: SessionState::Locked,
            last_activity: None,
            config: SessionConfig::default(),
            failed_attempts: 0,
            lockout_start: None,
            vault_file: None,
            encryption_key: None,
            session_start: None,
        }
    }

    /// Create a new session manager with custom configuration
    pub fn with_config(config: SessionConfig) -> Self {
        Self {
            state: SessionState::Locked,
            last_activity: None,
            config,
            failed_attempts: 0,
            lockout_start: None,
            vault_file: None,
            encryption_key: None,
            session_start: None,
        }
    }

    /// Get the current session state
    pub fn state(&self) -> &SessionState {
        &self.state
    }

    /// Check if session is unlocked
    pub fn is_unlocked(&self) -> bool {
        matches!(self.state, SessionState::Unlocked)
    }

    /// Check if session is locked
    pub fn is_locked(&self) -> bool {
        matches!(self.state, SessionState::Locked | SessionState::TimedOut)
    }

    /// Check if account is locked out
    pub fn is_locked_out(&self) -> bool {
        matches!(self.state, SessionState::LockedOut { .. })
    }

    /// Get remaining lockout time in seconds
    pub fn remaining_lockout_time(&self) -> Option<u64> {
        if let Some(start) = self.lockout_start {
            let lockout_duration = self.calculate_lockout_duration();
            let elapsed = start.elapsed().as_secs();
            if elapsed < lockout_duration {
                return Some(lockout_duration - elapsed);
            }
        }
        None
    }

    /// Get remaining lockout seconds (convenience method for GUI)
    pub fn lockout_remaining_secs(&self) -> u64 {
        self.remaining_lockout_time().unwrap_or(0)
    }

    /// Calculate lockout duration based on failed attempts (exponential backoff)
    fn calculate_lockout_duration(&self) -> u64 {
        // Exponential backoff: 30s, 60s, 120s, 240s, etc.
        BASE_LOCKOUT_SECS * (2u64.pow(self.failed_attempts.saturating_sub(self.config.max_failed_attempts)))
    }

    /// Record a failed login attempt
    pub fn record_failed_attempt(&mut self) {
        self.failed_attempts += 1;
        log::warn!("Failed login attempt {} of {}", self.failed_attempts, self.config.max_failed_attempts);

        if self.failed_attempts >= self.config.max_failed_attempts {
            self.lockout_start = Some(Instant::now());
            let duration = self.calculate_lockout_duration();
            self.state = SessionState::LockedOut { remaining_secs: duration };
            log::warn!("Account locked out for {} seconds", duration);
        }
    }

    /// Reset failed attempts on successful login (simple version for GUI)
    pub fn record_successful_login(&mut self) {
        self.failed_attempts = 0;
        self.lockout_start = None;
        self.state = SessionState::Unlocked;
        self.touch();
    }

    /// Record a successful login with full session data
    pub fn record_successful_login_with_key(&mut self, vault_file: &str, key: Key) {
        self.failed_attempts = 0;
        self.lockout_start = None;
        self.vault_file = Some(vault_file.to_string());
        self.encryption_key = Some(key);
        self.session_start = Some(Instant::now());
        self.state = SessionState::Unlocked;
        self.touch();
        log::info!("Session started for vault: {}", vault_file);
    }

    /// Update last activity timestamp (call on any user interaction)
    pub fn touch(&mut self) {
        self.last_activity = Some(Instant::now());
    }

    /// Check for timeout and update state if needed
    /// Returns true if session was timed out
    pub fn check_timeout(&mut self) -> bool {
        // Check lockout expiry
        if let SessionState::LockedOut { .. } = self.state {
            if let Some(remaining) = self.remaining_lockout_time() {
                self.state = SessionState::LockedOut { remaining_secs: remaining };
                return false;
            } else {
                // Lockout expired
                self.state = SessionState::Locked;
                self.lockout_start = None;
                log::info!("Lockout period expired");
                return false;
            }
        }

        // Skip if not unlocked or timeout disabled
        if self.config.lock_timeout_secs == 0 || !self.is_unlocked() {
            return false;
        }

        if let Some(last) = self.last_activity {
            let timeout = Duration::from_secs(self.config.lock_timeout_secs);
            if last.elapsed() >= timeout {
                self.timeout();
                return true;
            }
        }

        false
    }

    /// Get time until timeout (in seconds)
    pub fn time_until_timeout(&self) -> Option<u64> {
        if self.config.lock_timeout_secs == 0 || !self.is_unlocked() {
            return None;
        }

        if let Some(last) = self.last_activity {
            let elapsed = last.elapsed().as_secs();
            if elapsed < self.config.lock_timeout_secs {
                return Some(self.config.lock_timeout_secs - elapsed);
            }
        }

        Some(0)
    }

    /// Lock the session due to timeout
    fn timeout(&mut self) {
        log::info!("Session timed out after {} seconds of inactivity", self.config.lock_timeout_secs);
        self.state = SessionState::TimedOut;
        self.clear_sensitive_data();
    }

    /// Manually lock the session
    pub fn lock(&mut self) {
        if self.is_unlocked() {
            log::info!("Session locked manually");
            self.state = SessionState::Locked;
            self.clear_sensitive_data();
        }
    }

    /// Clear sensitive data from memory
    fn clear_sensitive_data(&mut self) {
        // Key will be zeroized on drop
        self.encryption_key = None;
        self.session_start = None;
        
        // Clear clipboard if configured
        if self.config.clear_clipboard_on_lock {
            if let Ok(clipboard) = crate::secure_clipboard::get_clipboard().lock() {
                let _ = clipboard.clear_now();
            }
        }
    }

    /// Get the current vault file
    pub fn vault_file(&self) -> Option<&str> {
        self.vault_file.as_deref()
    }

    /// Get the encryption key (only if unlocked)
    pub fn get_key(&self) -> Option<&Key> {
        if self.is_unlocked() {
            self.encryption_key.as_ref()
        } else {
            None
        }
    }

    /// Get session duration
    pub fn session_duration(&self) -> Option<Duration> {
        self.session_start.map(|start| start.elapsed())
    }

    /// Get session configuration
    pub fn config(&self) -> &SessionConfig {
        &self.config
    }

    /// Update session configuration
    pub fn set_config(&mut self, config: SessionConfig) {
        self.config = config;
    }

    /// Set lock timeout
    pub fn set_lock_timeout(&mut self, timeout_secs: u64) {
        self.config.lock_timeout_secs = timeout_secs;
    }

    /// Reset failed attempts counter (use after successful unlock)
    pub fn reset_failed_attempts(&mut self) {
        self.failed_attempts = 0;
        self.lockout_start = None;
        if matches!(self.state, SessionState::LockedOut { .. }) {
            self.state = SessionState::Locked;
        }
    }

    /// Get number of failed attempts
    pub fn failed_attempts(&self) -> u32 {
        self.failed_attempts
    }

    /// Get remaining attempts before lockout
    pub fn remaining_attempts(&self) -> u32 {
        self.config.max_failed_attempts.saturating_sub(self.failed_attempts)
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SessionManager {
    fn drop(&mut self) {
        // Ensure sensitive data is cleared
        self.clear_sensitive_data();
    }
}

/// Preset timeout configurations
pub mod presets {
    use super::SessionConfig;

    /// Quick lock (1 minute)
    pub fn quick_lock() -> SessionConfig {
        SessionConfig {
            lock_timeout_secs: 60,
            ..Default::default()
        }
    }

    /// Standard lock (5 minutes)
    pub fn standard() -> SessionConfig {
        SessionConfig::default()
    }

    /// Extended lock (15 minutes)
    pub fn extended() -> SessionConfig {
        SessionConfig {
            lock_timeout_secs: 900,
            ..Default::default()
        }
    }

    /// Never lock (not recommended)
    pub fn never_lock() -> SessionConfig {
        SessionConfig {
            lock_timeout_secs: 0,
            ..Default::default()
        }
    }

    /// High security (30 seconds, lock on minimize)
    pub fn high_security() -> SessionConfig {
        SessionConfig {
            lock_timeout_secs: 30,
            clear_clipboard_on_lock: true,
            lock_on_minimize: true,
            lock_on_screen_lock: true,
            max_failed_attempts: 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = SessionManager::new();
        assert!(session.is_locked());
        assert!(!session.is_unlocked());
    }

    #[test]
    fn test_failed_attempts() {
        let mut session = SessionManager::new();
        
        for i in 1..=MAX_FAILED_ATTEMPTS {
            session.record_failed_attempt();
            if i < MAX_FAILED_ATTEMPTS {
                assert!(!session.is_locked_out());
            }
        }
        
        assert!(session.is_locked_out());
    }

    #[test]
    fn test_remaining_attempts() {
        let mut session = SessionManager::new();
        assert_eq!(session.remaining_attempts(), MAX_FAILED_ATTEMPTS);
        
        session.record_failed_attempt();
        assert_eq!(session.remaining_attempts(), MAX_FAILED_ATTEMPTS - 1);
    }

    #[test]
    fn test_presets() {
        let quick = presets::quick_lock();
        assert_eq!(quick.lock_timeout_secs, 60);

        let high = presets::high_security();
        assert_eq!(high.lock_timeout_secs, 30);
        assert!(high.lock_on_minimize);
    }
}

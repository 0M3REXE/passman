//! Session Manager Module
//!
//! Handles session state, failed login lockout, and security policies
//! for the password manager application.

use std::time::Instant;

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
    /// Account is locked due to failed attempts
    LockedOut { remaining_secs: u64 },
}

/// Configuration for session behavior
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Maximum failed login attempts before lockout
    pub max_failed_attempts: u32,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_failed_attempts: MAX_FAILED_ATTEMPTS,
        }
    }
}

/// Session manager for handling authentication state and lockout
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
        }
    }

    /// Check if account is locked out
    pub fn is_locked_out(&self) -> bool {
        matches!(self.state, SessionState::LockedOut { .. })
    }

    /// Get remaining lockout time in seconds
    fn remaining_lockout_time(&self) -> Option<u64> {
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
        log::warn!(
            "Failed login attempt {} of {}",
            self.failed_attempts,
            self.config.max_failed_attempts
        );

        if self.failed_attempts >= self.config.max_failed_attempts {
            self.lockout_start = Some(Instant::now());
            let duration = self.calculate_lockout_duration();
            self.state = SessionState::LockedOut {
                remaining_secs: duration,
            };
            log::warn!("Account locked out for {} seconds", duration);
        }
    }

    /// Reset failed attempts on successful login
    pub fn record_successful_login(&mut self) {
        self.failed_attempts = 0;
        self.lockout_start = None;
        self.state = SessionState::Unlocked;
        self.touch();
    }

    /// Update last activity timestamp
    fn touch(&mut self) {
        self.last_activity = Some(Instant::now());
    }

    /// Get remaining attempts before lockout
    pub fn remaining_attempts(&self) -> u32 {
        self.config
            .max_failed_attempts
            .saturating_sub(self.failed_attempts)
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = SessionManager::new();
        assert!(!session.is_locked_out());
        assert_eq!(session.remaining_attempts(), MAX_FAILED_ATTEMPTS);
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
    fn test_successful_login_resets() {
        let mut session = SessionManager::new();

        session.record_failed_attempt();
        session.record_failed_attempt();
        assert_eq!(session.remaining_attempts(), MAX_FAILED_ATTEMPTS - 2);

        session.record_successful_login();
        assert_eq!(session.remaining_attempts(), MAX_FAILED_ATTEMPTS);
        assert!(!session.is_locked_out());
    }
}

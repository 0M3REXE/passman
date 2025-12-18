//! Configuration Module
//! 
//! Handles application configuration loading, saving, and defaults.
//! Configuration is stored in TOML format.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Default config filename
const CONFIG_FILE: &str = "passman.toml";

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// General settings
    #[serde(default)]
    pub general: GeneralConfig,
    
    /// Security settings
    #[serde(default)]
    pub security: SecurityConfig,
    
    /// UI settings
    #[serde(default)]
    pub ui: UiConfig,
    
    /// Password generation settings
    #[serde(default)]
    pub password: PasswordConfig,
    
    /// Backup settings
    #[serde(default)]
    pub backup: BackupConfig,
}

/// General application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Default vault file path
    #[serde(default = "default_vault_file")]
    pub default_vault: String,
    
    /// Enable logging
    #[serde(default = "default_true")]
    pub enable_logging: bool,
    
    /// Log level (error, warn, info, debug, trace)
    #[serde(default = "default_log_level")]
    pub log_level: String,
    
    /// Check for updates on startup
    #[serde(default)]
    pub check_updates: bool,
}

/// Security settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Auto-lock timeout in seconds (0 = disabled)
    #[serde(default = "default_lock_timeout")]
    pub lock_timeout_secs: u64,
    
    /// Clipboard auto-clear timeout in seconds (0 = disabled)
    #[serde(default = "default_clipboard_timeout")]
    pub clipboard_timeout_secs: u64,
    
    /// Clear clipboard on lock
    #[serde(default = "default_true")]
    pub clear_clipboard_on_lock: bool,
    
    /// Lock on window minimize
    #[serde(default)]
    pub lock_on_minimize: bool,
    
    /// Maximum failed login attempts before lockout
    #[serde(default = "default_max_attempts")]
    pub max_failed_attempts: u32,
    
    /// Minimum master password length
    #[serde(default = "default_min_password_length")]
    pub min_password_length: usize,
    
    /// Require uppercase in master password
    #[serde(default = "default_true")]
    pub require_uppercase: bool,
    
    /// Require lowercase in master password
    #[serde(default = "default_true")]
    pub require_lowercase: bool,
    
    /// Require numbers in master password
    #[serde(default = "default_true")]
    pub require_numbers: bool,
    
    /// Require symbols in master password
    #[serde(default)]
    pub require_symbols: bool,
    
    /// Argon2 memory cost in KB
    #[serde(default = "default_argon2_memory")]
    pub argon2_memory_kb: u32,
    
    /// Argon2 time cost (iterations)
    #[serde(default = "default_argon2_time")]
    pub argon2_time_cost: u32,
    
    /// Argon2 parallelism
    #[serde(default = "default_argon2_parallelism")]
    pub argon2_parallelism: u32,
}

/// UI settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Theme (dark, light, system)
    #[serde(default = "default_theme")]
    pub theme: String,
    
    /// Show password strength indicator
    #[serde(default = "default_true")]
    pub show_password_strength: bool,
    
    /// Show password health warnings
    #[serde(default = "default_true")]
    pub show_health_warnings: bool,
    
    /// Default sort order (name, created, modified)
    #[serde(default = "default_sort_order")]
    pub default_sort: String,
    
    /// Show entry icons
    #[serde(default = "default_true")]
    pub show_icons: bool,
    
    /// Compact list view
    #[serde(default)]
    pub compact_view: bool,
    
    /// Window width
    #[serde(default = "default_window_width")]
    pub window_width: f32,
    
    /// Window height
    #[serde(default = "default_window_height")]
    pub window_height: f32,
    
    /// Remember window position
    #[serde(default = "default_true")]
    pub remember_window_position: bool,
}

/// Password generation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordConfig {
    /// Default password length
    #[serde(default = "default_password_length")]
    pub default_length: usize,
    
    /// Include uppercase letters
    #[serde(default = "default_true")]
    pub include_uppercase: bool,
    
    /// Include lowercase letters
    #[serde(default = "default_true")]
    pub include_lowercase: bool,
    
    /// Include numbers
    #[serde(default = "default_true")]
    pub include_numbers: bool,
    
    /// Include symbols
    #[serde(default = "default_true")]
    pub include_symbols: bool,
    
    /// Exclude ambiguous characters (0, O, l, I)
    #[serde(default)]
    pub exclude_ambiguous: bool,
    
    /// Custom symbol set (if empty, use default)
    #[serde(default)]
    pub custom_symbols: String,
    
    /// Number of words for memorable passwords
    #[serde(default = "default_word_count")]
    pub memorable_word_count: usize,
}

/// Backup settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Enable automatic backups
    #[serde(default = "default_true")]
    pub auto_backup: bool,
    
    /// Backup directory (empty = same as vault)
    #[serde(default)]
    pub backup_directory: String,
    
    /// Maximum number of backups to keep
    #[serde(default = "default_max_backups")]
    pub max_backups: usize,
    
    /// Create backup before each save
    #[serde(default = "default_true")]
    pub backup_on_save: bool,
}

// Default value functions
fn default_vault_file() -> String { "vault.dat".to_string() }
fn default_true() -> bool { true }
fn default_log_level() -> String { "info".to_string() }
fn default_lock_timeout() -> u64 { 300 } // 5 minutes
fn default_clipboard_timeout() -> u64 { 30 }
fn default_max_attempts() -> u32 { 5 }
fn default_min_password_length() -> usize { 12 }
fn default_argon2_memory() -> u32 { 65536 } // 64 MB
fn default_argon2_time() -> u32 { 3 }
fn default_argon2_parallelism() -> u32 { 4 }
fn default_theme() -> String { "dark".to_string() }
fn default_sort_order() -> String { "name".to_string() }
fn default_window_width() -> f32 { 900.0 }
fn default_window_height() -> f32 { 650.0 }
fn default_password_length() -> usize { 20 }
fn default_word_count() -> usize { 4 }
fn default_max_backups() -> usize { 10 }

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            default_vault: default_vault_file(),
            enable_logging: true,
            log_level: default_log_level(),
            check_updates: false,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            lock_timeout_secs: default_lock_timeout(),
            clipboard_timeout_secs: default_clipboard_timeout(),
            clear_clipboard_on_lock: true,
            lock_on_minimize: false,
            max_failed_attempts: default_max_attempts(),
            min_password_length: default_min_password_length(),
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_symbols: false,
            argon2_memory_kb: default_argon2_memory(),
            argon2_time_cost: default_argon2_time(),
            argon2_parallelism: default_argon2_parallelism(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            show_password_strength: true,
            show_health_warnings: true,
            default_sort: default_sort_order(),
            show_icons: true,
            compact_view: false,
            window_width: default_window_width(),
            window_height: default_window_height(),
            remember_window_position: true,
        }
    }
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            default_length: default_password_length(),
            include_uppercase: true,
            include_lowercase: true,
            include_numbers: true,
            include_symbols: true,
            exclude_ambiguous: false,
            custom_symbols: String::new(),
            memorable_word_count: default_word_count(),
        }
    }
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            auto_backup: true,
            backup_directory: String::new(),
            max_backups: default_max_backups(),
            backup_on_save: true,
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn load() -> Self {
        Self::load_from(Self::config_path())
    }

    /// Load configuration from specific path
    pub fn load_from(path: PathBuf) -> Self {
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(contents) => {
                    match toml::from_str(&contents) {
                        Ok(config) => {
                            log::info!("Configuration loaded from {:?}", path);
                            return config;
                        }
                        Err(e) => {
                            log::warn!("Failed to parse config file: {}. Using defaults.", e);
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to read config file: {}. Using defaults.", e);
                }
            }
        } else {
            log::info!("No config file found. Using defaults.");
        }
        
        Self::default()
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<(), String> {
        self.save_to(Self::config_path())
    }

    /// Save configuration to specific path
    pub fn save_to(&self, path: PathBuf) -> Result<(), String> {
        let contents = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        
        fs::write(&path, contents)
            .map_err(|e| format!("Failed to write config file: {}", e))?;
        
        log::info!("Configuration saved to {:?}", path);
        Ok(())
    }

    /// Get default config file path
    pub fn config_path() -> PathBuf {
        // Try to use the app data directory, fallback to current directory
        if let Some(config_dir) = dirs::config_dir() {
            let app_dir = config_dir.join("passman");
            if !app_dir.exists() {
                let _ = fs::create_dir_all(&app_dir);
            }
            app_dir.join(CONFIG_FILE)
        } else {
            PathBuf::from(CONFIG_FILE)
        }
    }

    /// Validate master password against security requirements
    pub fn validate_master_password(&self, password: &str) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        if password.len() < self.security.min_password_length {
            errors.push(format!(
                "Password must be at least {} characters long",
                self.security.min_password_length
            ));
        }
        
        if self.security.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            errors.push("Password must contain at least one uppercase letter".to_string());
        }
        
        if self.security.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            errors.push("Password must contain at least one lowercase letter".to_string());
        }
        
        if self.security.require_numbers && !password.chars().any(|c| c.is_numeric()) {
            errors.push("Password must contain at least one number".to_string());
        }
        
        if self.security.require_symbols && !password.chars().any(|c| !c.is_alphanumeric()) {
            errors.push("Password must contain at least one symbol".to_string());
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Global configuration instance
static CONFIG: std::sync::OnceLock<std::sync::RwLock<Config>> = std::sync::OnceLock::new();

/// Get the global configuration (read-only)
pub fn get_config() -> std::sync::RwLockReadGuard<'static, Config> {
    CONFIG
        .get_or_init(|| std::sync::RwLock::new(Config::load()))
        .read()
        .expect("Config lock poisoned")
}

/// Get the global configuration (mutable)
pub fn get_config_mut() -> std::sync::RwLockWriteGuard<'static, Config> {
    CONFIG
        .get_or_init(|| std::sync::RwLock::new(Config::load()))
        .write()
        .expect("Config lock poisoned")
}

/// Reload configuration from file
pub fn reload_config() {
    let mut config = get_config_mut();
    *config = Config::load();
}

/// Save current configuration
pub fn save_config() -> Result<(), String> {
    get_config().save()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.security.lock_timeout_secs, 300);
        assert_eq!(config.security.clipboard_timeout_secs, 30);
        assert_eq!(config.password.default_length, 20);
    }

    #[test]
    fn test_password_validation() {
        let config = Config::default();
        
        // Too short
        assert!(config.validate_master_password("short").is_err());
        
        // Missing uppercase
        assert!(config.validate_master_password("lowercaseonly123").is_err());
        
        // Valid password
        assert!(config.validate_master_password("ValidPassword123").is_ok());
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        
        assert_eq!(config.security.lock_timeout_secs, parsed.security.lock_timeout_secs);
    }
}

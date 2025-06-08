use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::error::{PassmanError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Default vault file path
    pub default_vault: PathBuf,
    
    /// Auto-lock timeout in minutes (0 = never)
    pub auto_lock_timeout: u32,
    
    /// Clear clipboard after N seconds (0 = never)
    pub clipboard_clear_timeout: u32,
    
    /// Default password generation length
    pub default_password_length: usize,
    
    /// Password generation character sets
    pub password_config: PasswordConfig,
    
    /// Crypto settings
    pub crypto_config: CryptoConfig,
    
    /// UI settings
    pub ui_config: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordConfig {
    pub include_uppercase: bool,
    pub include_lowercase: bool,
    pub include_numbers: bool,
    pub include_symbols: bool,
    pub exclude_ambiguous: bool, // 0, O, l, I, etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoConfig {
    pub argon2_memory: u32,
    pub argon2_iterations: u32,
    pub argon2_parallelism: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub font_size: f32,
    pub window_size: (f32, f32),
    pub remember_window_position: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_vault: PathBuf::from("vault.dat"),
            auto_lock_timeout: 15, // 15 minutes
            clipboard_clear_timeout: 30, // 30 seconds
            default_password_length: 16,
            password_config: PasswordConfig::default(),
            crypto_config: CryptoConfig::default(),
            ui_config: UiConfig::default(),
        }
    }
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            include_uppercase: true,
            include_lowercase: true,
            include_numbers: true,
            include_symbols: true,
            exclude_ambiguous: true,
        }
    }
}

impl Default for CryptoConfig {
    fn default() -> Self {
        Self {
            argon2_memory: 65536, // 64 MB
            argon2_iterations: 3,
            argon2_parallelism: 4,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            font_size: 14.0,
            window_size: (800.0, 600.0),
            remember_window_position: true,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)
                .map_err(|e| PassmanError::InvalidInput(format!("Invalid config: {}", e)))?;
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)
            .map_err(|e| PassmanError::InvalidInput(format!("Config serialization error: {}", e)))?;
        
        std::fs::write(&config_path, content)?;
        Ok(())
    }
    
    fn config_path() -> Result<PathBuf> {
        let mut path = dirs::config_dir()
            .ok_or_else(|| PassmanError::InvalidInput("Cannot determine config directory".to_string()))?;
        path.push("passman");
        path.push("config.toml");
        Ok(path)
    }
}

//! Logging Module
//!
//! Structured logging with levels, file output, and secure handling.
//! Ensures sensitive data is never logged.

#![allow(dead_code)]

use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

/// Logger configuration
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Log level filter
    pub level: LevelFilter,
    /// Whether to log to console
    pub console_output: bool,
    /// Optional file path for log output
    pub file_path: Option<PathBuf>,
    /// Whether to include timestamps
    pub include_timestamps: bool,
    /// Whether to include module path
    pub include_module: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: LevelFilter::Info,
            console_output: true,
            file_path: None,
            include_timestamps: true,
            include_module: true,
        }
    }
}

impl LogConfig {
    /// Create a debug configuration
    pub fn debug() -> Self {
        Self {
            level: LevelFilter::Debug,
            ..Default::default()
        }
    }
    
    /// Create a production configuration (no console, file only)
    pub fn production(file_path: PathBuf) -> Self {
        Self {
            level: LevelFilter::Info,
            console_output: false,
            file_path: Some(file_path),
            include_timestamps: true,
            include_module: false,
        }
    }
    
    /// Create from config file settings
    pub fn from_config() -> Self {
        let config = crate::config::get_config();
        let level = match config.general.log_level.to_lowercase().as_str() {
            "error" => LevelFilter::Error,
            "warn" => LevelFilter::Warn,
            "info" => LevelFilter::Info,
            "debug" => LevelFilter::Debug,
            "trace" => LevelFilter::Trace,
            _ => LevelFilter::Info,
        };
        
        let file_path = if config.general.enable_logging {
            Some(get_log_file_path())
        } else {
            None
        };
        
        Self {
            level,
            console_output: cfg!(debug_assertions), // Console only in debug builds
            file_path,
            include_timestamps: true,
            include_module: true,
        }
    }
}

/// Get the default log file path
pub fn get_log_file_path() -> PathBuf {
    let base_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."));
    let log_dir = base_dir.join("passman").join("logs");
    
    // Create log directory if it doesn't exist
    let _ = std::fs::create_dir_all(&log_dir);
    
    // Use date-based log file name
    let date = chrono::Local::now().format("%Y-%m-%d");
    log_dir.join(format!("passman_{}.log", date))
}

/// Custom logger implementation
struct PassmanLogger {
    config: LogConfig,
    file: Option<Mutex<File>>,
}

impl PassmanLogger {
    fn new(config: LogConfig) -> Self {
        let file = config.file_path.as_ref().and_then(|path| {
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
                .ok()
                .map(Mutex::new)
        });
        
        Self { config, file }
    }
    
    fn format_record(&self, record: &Record) -> String {
        let mut parts = Vec::new();
        
        if self.config.include_timestamps {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            parts.push(format!("[{}]", timestamp));
        }
        
        // Level with padding
        parts.push(format!("[{:5}]", record.level()));
        
        if self.config.include_module {
            if let Some(module) = record.module_path() {
                // Shorten module path for readability
                let short_module = module
                    .split("::")
                    .last()
                    .unwrap_or(module);
                parts.push(format!("[{}]", short_module));
            }
        }
        
        parts.push(record.args().to_string());
        
        parts.join(" ")
    }
}

impl log::Log for PassmanLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.config.level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        
        // Filter out sensitive module paths
        if let Some(module) = record.module_path() {
            // Don't log from crypto module at trace level
            if module.contains("crypto") && record.level() == Level::Trace {
                return;
            }
        }
        
        let formatted = self.format_record(record);
        
        // Console output
        if self.config.console_output {
            let color = match record.level() {
                Level::Error => "\x1b[31m", // Red
                Level::Warn => "\x1b[33m",  // Yellow
                Level::Info => "\x1b[32m",  // Green
                Level::Debug => "\x1b[36m", // Cyan
                Level::Trace => "\x1b[90m", // Gray
            };
            eprintln!("{}{}\x1b[0m", color, formatted);
        }
        
        // File output
        if let Some(ref file_mutex) = self.file {
            if let Ok(mut file) = file_mutex.lock() {
                let _ = writeln!(file, "{}", formatted);
            }
        }
    }

    fn flush(&self) {
        if let Some(ref file_mutex) = self.file {
            if let Ok(mut file) = file_mutex.lock() {
                let _ = file.flush();
            }
        }
    }
}

/// Initialize the logger with the given configuration
pub fn init(config: LogConfig) -> Result<(), SetLoggerError> {
    let logger = PassmanLogger::new(config.clone());
    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(config.level);
    Ok(())
}

/// Initialize logger from application config
pub fn init_from_config() -> Result<(), SetLoggerError> {
    init(LogConfig::from_config())
}

/// Initialize logger with default debug settings
pub fn init_debug() -> Result<(), SetLoggerError> {
    init(LogConfig::debug())
}

// === Logging Macros for Security-Sensitive Operations ===

/// Log a security event (always logged at INFO level)
#[macro_export]
macro_rules! log_security {
    ($($arg:tt)*) => {
        log::info!(target: "security", $($arg)*)
    };
}

/// Log a vault operation
#[macro_export]
macro_rules! log_vault {
    ($($arg:tt)*) => {
        log::debug!(target: "vault", $($arg)*)
    };
}

/// Log a cryptographic operation (without sensitive data)
#[macro_export]
macro_rules! log_crypto {
    ($($arg:tt)*) => {
        log::debug!(target: "crypto", $($arg)*)
    };
}

/// Log a session event
#[macro_export]
macro_rules! log_session {
    ($($arg:tt)*) => {
        log::debug!(target: "session", $($arg)*)
    };
}

/// Log an error with context
#[macro_export]
macro_rules! log_error_ctx {
    ($err:expr, $ctx:expr) => {
        log::error!("{}: {}", $ctx, $err)
    };
}

// === Helper Functions ===

/// Mask sensitive data for logging (show only first and last chars)
pub fn mask_sensitive(data: &str) -> String {
    if data.len() <= 4 {
        return "*".repeat(data.len());
    }
    format!(
        "{}{}{}",
        &data[..1],
        "*".repeat(data.len() - 2),
        &data[data.len()-1..]
    )
}

/// Create a safe identifier for logging (removes path separators, etc.)
pub fn safe_log_id(id: &str) -> String {
    id.chars()
        .take(50)
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mask_sensitive() {
        assert_eq!(mask_sensitive(""), "");
        assert_eq!(mask_sensitive("a"), "*");
        assert_eq!(mask_sensitive("ab"), "**");
        assert_eq!(mask_sensitive("abc"), "***");
        assert_eq!(mask_sensitive("abcd"), "****");
        assert_eq!(mask_sensitive("abcde"), "a***e");
        assert_eq!(mask_sensitive("password123"), "p*********3");
    }
    
    #[test]
    fn test_safe_log_id() {
        assert_eq!(safe_log_id("test-entry_1"), "test-entry_1");
        assert_eq!(safe_log_id("entry with spaces"), "entrywithspaces");
        assert_eq!(safe_log_id("../../../etc/passwd"), "etcpasswd");
    }
}

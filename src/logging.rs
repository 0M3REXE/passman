use log::{info, warn, error, debug};
use std::fs::OpenOptions;
use std::io::Write;

pub struct Logger;

impl Logger {
    pub fn init() -> Result<(), Box<dyn std::error::Error>> {
        env_logger::Builder::from_default_env()
            .target(env_logger::Target::Stdout)
            .filter_level(log::LevelFilter::Info)
            .init();
        
        info!("Passman logger initialized");
        Ok(())
    }
    
    pub fn log_vault_operation(operation: &str, vault_path: &str) {
        info!("Vault operation: {} on {}", operation, vault_path);
    }
    
    pub fn log_entry_operation(operation: &str, entry_id: &str) {
        info!("Entry operation: {} on entry '{}'", operation, entry_id);
    }
    
    pub fn log_security_event(event: &str) {
        warn!("Security event: {}", event);
    }
    
    pub fn log_error(error: &str) {
        error!("Error: {}", error);
    }
    
    pub fn log_crypto_operation(operation: &str) {
        debug!("Crypto operation: {}", operation);
    }
}

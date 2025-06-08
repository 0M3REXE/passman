use crate::error::{PassmanError, Result};
use std::path::{Path, PathBuf};
use std::fs;
use chrono::{DateTime, Utc};

pub struct BackupManager;

impl BackupManager {
    /// Create a backup of the vault file
    pub fn create_backup(vault_path: &str) -> Result<PathBuf> {
        let vault_path = Path::new(vault_path);
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("{}.backup.{}", 
            vault_path.file_stem().unwrap_or_default().to_string_lossy(),
            timestamp
        );
        
        let backup_dir = vault_path.parent().unwrap_or(Path::new(".")).join("backups");
        fs::create_dir_all(&backup_dir)?;
        
        let backup_path = backup_dir.join(backup_name);
        fs::copy(vault_path, &backup_path)?;
        
        crate::logging::Logger::log_vault_operation("backup_created", &backup_path.to_string_lossy());
        Ok(backup_path)
    }
    
    /// List available backups for a vault
    pub fn list_backups(vault_path: &str) -> Result<Vec<BackupInfo>> {
        let vault_path = Path::new(vault_path);
        let backup_dir = vault_path.parent().unwrap_or(Path::new(".")).join("backups");
        
        if !backup_dir.exists() {
            return Ok(Vec::new());
        }
        
        let vault_stem = vault_path.file_stem().unwrap_or_default().to_string_lossy();
        let mut backups = Vec::new();
        
        for entry in fs::read_dir(&backup_dir)? {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            
            if file_name.starts_with(&vault_stem.to_string()) && file_name.contains(".backup.") {
                let metadata = entry.metadata()?;
                let created = metadata.created()
                    .map(|time| DateTime::<Utc>::from(time))
                    .unwrap_or_else(|_| Utc::now());
                
                backups.push(BackupInfo {
                    path: entry.path(),
                    created,
                    size: metadata.len(),
                });
            }
        }
        
        backups.sort_by(|a, b| b.created.cmp(&a.created)); // newest first
        Ok(backups)
    }
    
    /// Restore from a backup
    pub fn restore_backup(backup_path: &Path, vault_path: &str) -> Result<()> {
        if !backup_path.exists() {
            return Err(PassmanError::VaultNotFound(backup_path.to_string_lossy().to_string()));
        }
        
        // Create backup of current vault before restoring
        if Path::new(vault_path).exists() {
            Self::create_backup(vault_path)?;
        }
        
        fs::copy(backup_path, vault_path)?;
        
        crate::logging::Logger::log_vault_operation("backup_restored", vault_path);
        Ok(())
    }
    
    /// Clean old backups (keep only N most recent)
    pub fn cleanup_backups(vault_path: &str, keep_count: usize) -> Result<usize> {
        let backups = Self::list_backups(vault_path)?;
        let to_remove = backups.iter().skip(keep_count);
        let mut removed_count = 0;
        
        for backup in to_remove {
            if fs::remove_file(&backup.path).is_ok() {
                removed_count += 1;
            }
        }
        
        Ok(removed_count)
    }
}

#[derive(Debug, Clone)]
pub struct BackupInfo {
    pub path: PathBuf,
    pub created: DateTime<Utc>,
    pub size: u64,
}

impl BackupInfo {
    pub fn display_name(&self) -> String {
        format!("{} ({} bytes)", 
            self.created.format("%Y-%m-%d %H:%M:%S"),
            self.size
        )
    }
}

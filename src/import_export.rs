use crate::model::{Entry, Vault};
use crate::vault::VaultManager;
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{Read, Write};
use zeroize::Zeroizing;

#[derive(Serialize, Deserialize)]
struct ExportEntry {
    id: String,
    username: String,
    password: String,
    note: Option<String>,
    url: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    last_changed: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize)]
struct ExportData {
    version: String,
    exported_at: chrono::DateTime<chrono::Utc>,
    entries: Vec<ExportEntry>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct CsvEntry {
    #[serde(alias = "name", alias = "title", alias = "site")]
    id: String,
    #[serde(alias = "login", alias = "email")]
    username: String,
    password: String,
    #[serde(alias = "notes", alias = "comment")]
    note: Option<String>,
    #[serde(alias = "website")]
    url: Option<String>,
}

pub struct ImportExportManager;

impl ImportExportManager {
    /// Export vault to JSON format
    pub fn export_json(
        vault: &Vault,
        output_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let entries: Vec<ExportEntry> = vault
            .list_entries()
            .iter()
            .filter_map(|id| {                vault.get_entry(id).map(|entry| ExportEntry {
                    id: id.to_string(),
                    username: entry.username.clone(),
                    password: entry.password.clone(),
                    note: entry.note.clone(),
                    url: entry.url.clone(),
                    created_at: entry.created_at,
                    last_changed: entry.modified_at,
                })
            })
            .collect();

        let export_data = ExportData {
            version: "1.0".to_string(),
            exported_at: chrono::Utc::now(),
            entries,
        };

        let json = serde_json::to_string_pretty(&export_data)?;
        let mut file = File::create(output_path)?;
        file.write_all(json.as_bytes())?;

        println!("✓ Exported {} entries to {}", export_data.entries.len(), output_path);
        Ok(())
    }

    /// Export vault to CSV format
    pub fn export_csv(
        vault: &Vault,
        output_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create(output_path)?;
        writeln!(file, "id,username,password,note,url")?;

        let mut count = 0;
        for id in vault.list_entries() {
            if let Some(entry) = vault.get_entry(id) {
                let note = entry.note.as_deref().unwrap_or("");
                let url = ""; // TODO: Add URL field to Entry struct
                writeln!(
                    file,
                    "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"",
                    id.replace("\"", "\"\""),
                    entry.username.replace("\"", "\"\""),
                    entry.password.replace("\"", "\"\""),
                    note.replace("\"", "\"\""),
                    url
                )?;
                count += 1;
            }
        }

        println!("✓ Exported {} entries to {}", count, output_path);
        Ok(())
    }

    /// Import from JSON format
    pub fn import_json(
        input_path: &str,
        master_password: &Zeroizing<String>,
        vault_file: Option<&str>,
        merge: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::open(input_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let import_data: ExportData = serde_json::from_str(&contents)?;

        let mut vault = if merge && VaultManager::exists(vault_file) {
            VaultManager::load(master_password, vault_file)?
        } else {
            if VaultManager::exists(vault_file) && !merge {
                return Err("Vault already exists! Use --merge flag to merge with existing vault or choose a different vault file.".into());
            }
            Vault::new()
        };

        let mut imported_count = 0;
        let mut skipped_count = 0;

        for export_entry in import_data.entries {
            if vault.get_entry(&export_entry.id).is_some() {
                println!("⚠ Skipping existing entry: {}", export_entry.id);
                skipped_count += 1;
                continue;
            }

            let entry = Entry::new(
                export_entry.username,
                export_entry.password,
                export_entry.note,
            );

            vault.add_entry(export_entry.id.clone(), entry);
            imported_count += 1;
        }

        if !VaultManager::exists(vault_file) {
            VaultManager::init(master_password, vault_file)?;
        }
        VaultManager::save(&vault, master_password, vault_file)?;

        println!("✓ Import completed:");
        println!("  - Imported: {} entries", imported_count);
        if skipped_count > 0 {
            println!("  - Skipped: {} existing entries", skipped_count);
        }

        Ok(())
    }

    /// Import from CSV format
    pub fn import_csv(
        input_path: &str,
        master_password: &Zeroizing<String>,
        vault_file: Option<&str>,
        merge: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::open(input_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let mut vault = if merge && VaultManager::exists(vault_file) {
            VaultManager::load(master_password, vault_file)?
        } else {
            if VaultManager::exists(vault_file) && !merge {
                return Err("Vault already exists! Use --merge flag to merge with existing vault or choose a different vault file.".into());
            }
            Vault::new()
        };

        let mut reader = csv::Reader::from_reader(contents.as_bytes());
        let mut imported_count = 0;
        let mut skipped_count = 0;

        for result in reader.deserialize() {
            let csv_entry: CsvEntry = result?;

            if vault.get_entry(&csv_entry.id).is_some() {
                println!("⚠ Skipping existing entry: {}", csv_entry.id);
                skipped_count += 1;
                continue;
            }

            let entry = Entry::new(
                csv_entry.username,
                csv_entry.password,
                csv_entry.note,
            );

            vault.add_entry(csv_entry.id.clone(), entry);
            imported_count += 1;
        }

        if !VaultManager::exists(vault_file) {
            VaultManager::init(master_password, vault_file)?;
        }
        VaultManager::save(&vault, master_password, vault_file)?;

        println!("✓ Import completed:");
        println!("  - Imported: {} entries", imported_count);
        if skipped_count > 0 {
            println!("  - Skipped: {} existing entries", skipped_count);
        }

        Ok(())
    }

    /// Import from Chrome/Firefox format (basic JSON)
    pub fn import_browser(
        input_path: &str,
        master_password: &Zeroizing<String>,
        vault_file: Option<&str>,
        browser_type: &str,
        merge: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::open(input_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let json_data: serde_json::Value = serde_json::from_str(&contents)?;

        let mut vault = if merge && VaultManager::exists(vault_file) {
            VaultManager::load(master_password, vault_file)?
        } else {
            if VaultManager::exists(vault_file) && !merge {
                return Err("Vault already exists! Use --merge flag to merge with existing vault or choose a different vault file.".into());
            }
            Vault::new()
        };

        let mut imported_count = 0;
        let mut skipped_count = 0;

        // Handle Chrome export format
        if browser_type == "chrome" {
            if let Some(passwords) = json_data.get("passwords").and_then(|p| p.as_array()) {
                for password_entry in passwords {
                    if let (Some(origin), Some(username), Some(password)) = (
                        password_entry.get("origin").and_then(|o| o.as_str()),
                        password_entry.get("username").and_then(|u| u.as_str()),
                        password_entry.get("password").and_then(|p| p.as_str()),
                    ) {
                        let id = format!("{}_{}", origin, username);
                        
                        if vault.get_entry(&id).is_some() {
                            println!("⚠ Skipping existing entry: {}", id);
                            skipped_count += 1;
                            continue;
                        }

                        let entry = Entry::new(
                            username.to_string(),
                            password.to_string(),
                            Some(format!("Imported from Chrome: {}", origin)),
                        );

                        vault.add_entry(id, entry);
                        imported_count += 1;
                    }
                }
            }
        }

        if !VaultManager::exists(vault_file) {
            VaultManager::init(master_password, vault_file)?;
        }
        VaultManager::save(&vault, master_password, vault_file)?;

        println!("✓ Browser import completed:");
        println!("  - Imported: {} entries", imported_count);
        if skipped_count > 0 {
            println!("  - Skipped: {} existing entries", skipped_count);
        }

        Ok(())
    }

    /// Create automatic backup before risky operations
    pub fn create_auto_backup(vault_file: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
        VaultManager::create_backup(vault_file)
    }    /// List available backup files
    #[allow(dead_code)]
    pub fn list_backups() -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut backups = Vec::new();
        
        for entry in std::fs::read_dir(".")? {
            let entry = entry?;
            let path = entry.path();
            if let Some(name) = path.file_name() {
                if let Some(name_str) = name.to_str() {
                    if name_str.contains(".backup.") && name_str.ends_with(".dat") {
                        backups.push(name_str.to_string());
                    }
                }
            }
        }
        
        backups.sort_by(|a, b| b.cmp(a)); // Most recent first
        Ok(backups)
    }
}

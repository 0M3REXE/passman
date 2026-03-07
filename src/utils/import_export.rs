//! Import / Export Module
//!
//! Supports JSON, CSV, and browser (Chrome / Firefox) formats for importing
//! and exporting password entries.

use crate::core::error::{PassmanError, PassmanResult, TransferError};
use crate::core::model::{Entry, Vault};
use crate::security::vault::VaultManager;

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use zeroize::Zeroizing;

// ---------------------------------------------------------------------------
// Serialization models
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// ImportExportManager
// ---------------------------------------------------------------------------

pub struct ImportExportManager;

impl ImportExportManager {
    /// Export vault to JSON format.
    pub fn export_json(vault: &Vault, output_path: &str) -> PassmanResult<()> {
        let entries: Vec<ExportEntry> = vault
            .list_entries()
            .iter()
            .filter_map(|id| {
                vault.get_entry(id).map(|entry| ExportEntry {
                    id: id.to_string(),
                    username: entry.username.clone(),
                    password: entry.password_str().to_string(),
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
        let mut file = File::create(output_path).map_err(|e| {
            PassmanError::Transfer(TransferError::PermissionDenied(e.to_string()))
        })?;
        file.write_all(json.as_bytes())?;

        println!(
            "✓ Exported {} entries to {}",
            export_data.entries.len(),
            output_path
        );
        Ok(())
    }

    /// Export vault to CSV format (**WARNING**: plaintext passwords).
    pub fn export_csv(vault: &Vault, output_path: &str) -> PassmanResult<()> {
        let mut file = File::create(output_path).map_err(|e| {
            PassmanError::Transfer(TransferError::PermissionDenied(e.to_string()))
        })?;
        writeln!(file, "id,username,password,note,url")?;

        let mut count = 0;
        for id in vault.list_entries() {
            if let Some(entry) = vault.get_entry(id) {
                let note = entry.note.as_deref().unwrap_or("");
                let url = entry.url.as_deref().unwrap_or("");
                writeln!(
                    file,
                    "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"",
                    id.replace('"', "\"\""),
                    entry.username.replace('"', "\"\""),
                    entry.password_str().replace('"', "\"\""),
                    note.replace('"', "\"\""),
                    url.replace('"', "\"\""),
                )?;
                count += 1;
            }
        }

        println!("✓ Exported {} entries to {}", count, output_path);
        Ok(())
    }

    /// Import entries from a JSON file.
    pub fn import_json(
        input_path: &str,
        master_password: &Zeroizing<String>,
        vault_file: Option<&str>,
        merge: bool,
    ) -> PassmanResult<()> {
        let mut file = File::open(input_path).map_err(|_| {
            PassmanError::Transfer(TransferError::FileNotFound(input_path.to_string()))
        })?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let import_data: ExportData = serde_json::from_str(&contents)
            .map_err(|e| PassmanError::Transfer(TransferError::ParseError(e.to_string())))?;

        let mut vault = Self::prepare_vault(master_password, vault_file, merge)?;

        let (mut imported, mut skipped) = (0usize, 0usize);

        for export_entry in import_data.entries {
            if vault.get_entry(&export_entry.id).is_some() {
                println!("⚠ Skipping existing entry: {}", export_entry.id);
                skipped += 1;
                continue;
            }
            let entry = Entry::new(export_entry.username, export_entry.password, export_entry.note);
            vault.add_entry(export_entry.id, entry);
            imported += 1;
        }

        Self::finalize_vault(&vault, master_password, vault_file)?;
        Self::print_import_summary(imported, skipped);
        Ok(())
    }

    /// Import entries from a CSV file.
    pub fn import_csv(
        input_path: &str,
        master_password: &Zeroizing<String>,
        vault_file: Option<&str>,
        merge: bool,
    ) -> PassmanResult<()> {
        let mut file = File::open(input_path).map_err(|_| {
            PassmanError::Transfer(TransferError::FileNotFound(input_path.to_string()))
        })?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let mut vault = Self::prepare_vault(master_password, vault_file, merge)?;
        let mut reader = csv::Reader::from_reader(contents.as_bytes());

        let (mut imported, mut skipped) = (0usize, 0usize);

        for result in reader.deserialize() {
            let csv_entry: CsvEntry = result?;

            if vault.get_entry(&csv_entry.id).is_some() {
                println!("⚠ Skipping existing entry: {}", csv_entry.id);
                skipped += 1;
                continue;
            }

            let mut entry = Entry::new(csv_entry.username, csv_entry.password, csv_entry.note);
            entry.url = csv_entry.url;
            vault.add_entry(csv_entry.id, entry);
            imported += 1;
        }

        Self::finalize_vault(&vault, master_password, vault_file)?;
        Self::print_import_summary(imported, skipped);
        Ok(())
    }

    /// Import from Chrome or Firefox JSON export.
    pub fn import_browser(
        input_path: &str,
        master_password: &Zeroizing<String>,
        vault_file: Option<&str>,
        browser_type: &str,
        merge: bool,
    ) -> PassmanResult<()> {
        let mut file = File::open(input_path).map_err(|_| {
            PassmanError::Transfer(TransferError::FileNotFound(input_path.to_string()))
        })?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let json_data: serde_json::Value = serde_json::from_str(&contents)
            .map_err(|e| PassmanError::Transfer(TransferError::ParseError(e.to_string())))?;

        let mut vault = Self::prepare_vault(master_password, vault_file, merge)?;
        let (mut imported, mut skipped) = (0usize, 0usize);

        match browser_type {
            "chrome" => Self::parse_chrome(&json_data, &mut vault, &mut imported, &mut skipped),
            "firefox" => Self::parse_firefox(&json_data, &mut vault, &mut imported, &mut skipped),
            other => {
                return Err(PassmanError::Transfer(TransferError::UnsupportedFormat(
                    other.to_string(),
                )));
            }
        }

        Self::finalize_vault(&vault, master_password, vault_file)?;
        println!("✓ Browser import completed:");
        Self::print_import_summary(imported, skipped);
        Ok(())
    }

    // ----- private helpers -----

    /// Load or create the target vault depending on `merge`.
    fn prepare_vault(
        master_password: &Zeroizing<String>,
        vault_file: Option<&str>,
        merge: bool,
    ) -> PassmanResult<Vault> {
        if merge && VaultManager::exists(vault_file) {
            VaultManager::load(master_password, vault_file)
        } else {
            if VaultManager::exists(vault_file) && !merge {
                return Err(PassmanError::Transfer(TransferError::InvalidData(
                    "Vault already exists! Use --merge to merge with existing vault.".into(),
                )));
            }
            Ok(Vault::new())
        }
    }

    /// Ensure the vault file exists on disk and save.
    fn finalize_vault(
        vault: &Vault,
        master_password: &Zeroizing<String>,
        vault_file: Option<&str>,
    ) -> PassmanResult<()> {
        if !VaultManager::exists(vault_file) {
            VaultManager::init(master_password, vault_file)?;
        }
        VaultManager::save(vault, master_password, vault_file)
    }

    fn print_import_summary(imported: usize, skipped: usize) {
        println!("✓ Import completed:");
        println!("  - Imported: {} entries", imported);
        if skipped > 0 {
            println!("  - Skipped: {} existing entries", skipped);
        }
    }

    fn parse_chrome(
        json: &serde_json::Value,
        vault: &mut Vault,
        imported: &mut usize,
        skipped: &mut usize,
    ) {
        if let Some(passwords) = json.get("passwords").and_then(|p| p.as_array()) {
            for entry in passwords {
                if let (Some(origin), Some(username), Some(password)) = (
                    entry.get("origin").and_then(|o| o.as_str()),
                    entry.get("username").and_then(|u| u.as_str()),
                    entry.get("password").and_then(|p| p.as_str()),
                ) {
                    let id = format!("{}_{}", origin, username);
                    if vault.get_entry(&id).is_some() {
                        println!("⚠ Skipping existing entry: {}", id);
                        *skipped += 1;
                        continue;
                    }
                    let e = Entry::new(
                        username.to_string(),
                        password.to_string(),
                        Some(format!("Imported from Chrome: {}", origin)),
                    );
                    vault.add_entry(id, e);
                    *imported += 1;
                }
            }
        }
    }

    fn parse_firefox(
        json: &serde_json::Value,
        vault: &mut Vault,
        imported: &mut usize,
        skipped: &mut usize,
    ) {
        if let Some(logins) = json.get("logins").and_then(|l| l.as_array()) {
            for login in logins {
                if let (Some(hostname), Some(username), Some(password)) = (
                    login.get("hostname").and_then(|h| h.as_str()),
                    login
                        .get("encryptedUsername")
                        .and_then(|u| u.as_str())
                        .or_else(|| login.get("username").and_then(|u| u.as_str())),
                    login
                        .get("encryptedPassword")
                        .and_then(|p| p.as_str())
                        .or_else(|| login.get("password").and_then(|p| p.as_str())),
                ) {
                    let id = format!("{}_{}", hostname, username);
                    if vault.get_entry(&id).is_some() {
                        println!("⚠ Skipping existing entry: {}", id);
                        *skipped += 1;
                        continue;
                    }
                    let e = Entry::new(
                        username.to_string(),
                        password.to_string(),
                        Some(format!("Imported from Firefox: {}", hostname)),
                    );
                    vault.add_entry(id, e);
                    *imported += 1;
                }
            }
        }
    }
}

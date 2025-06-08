mod cli;
mod crypto;
mod vault;
mod model;
mod utils;
mod gui;
mod error;
mod config;
mod secure_memory;
mod two_factor;
mod health;
mod import_export;

use eframe::egui;
use cli::{Cli, Commands, TransferCommands, ConfigCommands};
use model::Entry;
use vault::VaultManager;
use utils::*;
use clap::Parser;
use std::error::Error;
use std::io::Write;
use zeroize::Zeroizing;

fn main() -> Result<(), eframe::Error> {
    // Check if CLI arguments are provided
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 {
        // Run CLI mode for backward compatibility
        run_cli();
        return Ok(());
    }

    // Run GUI mode
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0])
            .with_title("Passman - Password Manager")
            .with_icon(eframe::icon_data::from_png_bytes(&[]).unwrap_or_default()),
        ..Default::default()
    };

    eframe::run_native(
        "Passman",
        options,
        Box::new(|cc| Ok(Box::new(gui::PassmanApp::new(cc)))),
    )
}

fn run_cli() {
    let cli = Cli::parse();
    let vault_file = cli.vault.as_deref();    let result = match cli.command {
        Commands::Init { .. } => handle_init(vault_file),
        Commands::Add { id, .. } => handle_add(&id, vault_file),
        Commands::Get { id, .. } => handle_get(&id, vault_file),
        Commands::List { .. } => handle_list(vault_file),
        Commands::Edit { id } => handle_edit(&id, vault_file),
        Commands::Remove { id, .. } => handle_remove(&id, vault_file),
        Commands::Check { password, .. } => handle_check(password.as_deref()),
        Commands::Vaults => handle_vaults(),
        Commands::Generate { length, symbols, memorable, .. } => handle_generate(length, symbols, memorable),        Commands::Transfer(transfer_cmd) => handle_transfer(transfer_cmd, vault_file),
        Commands::Config(config_cmd) => handle_config(config_cmd),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn handle_init(vault_file: Option<&str>) -> Result<(), Box<dyn Error>> {
    if VaultManager::exists(vault_file) {
        return Err("Vault already exists! Remove vault file to reset.".into());
    }

    let master_password = read_password_secure("Create a master password: ")?;
    let confirm_password = read_password_secure("Confirm master password: ")?;

    if master_password.as_str() != confirm_password.as_str() {
        return Err("Passwords do not match!".into());
    }

    if master_password.len() < 8 {
        return Err("Master password must be at least 8 characters long!".into());
    }

    VaultManager::init(&master_password, vault_file)?;
    println!("✓ Vault initialized successfully!");
    Ok(())
}

fn handle_add(id: &str, vault_file: Option<&str>) -> Result<(), Box<dyn Error>> {
    let master_password = read_password_secure("Enter master password: ")?;
    let mut vault = VaultManager::load(&master_password, vault_file)?;

    if vault.get_entry(id).is_some() {
        return Err(format!("Entry '{}' already exists!", id).into());
    }

    println!("Adding new entry for '{}'", id);
    let username = read_line("Username: ")?;
    
    let password_choice = read_line_optional("Generate password? (y/N): ")?;
    let password = if password_choice.to_lowercase() == "y" || password_choice.to_lowercase() == "yes" {
        let generated = generate_password(16);
        println!("Generated password: {}", generated);
        let (strength, _) = analyze_password_strength(&generated);
        println!("Password strength: {}", strength);
        generated
    } else {
        let pwd = read_password_secure("Password: ")?;
        let (strength, suggestions) = analyze_password_strength(&pwd);
        println!("Password strength: {}", strength);
        if !suggestions.is_empty() {
            println!("Suggestions:");
            for suggestion in suggestions {
                println!("  • {}", suggestion);
            }
        }
        pwd.to_string()
    };

    let note_input = read_line_optional("Note (optional): ")?;
    let note = if note_input.is_empty() { None } else { Some(note_input) };

    let entry = Entry::new(username, password, note);
    vault.add_entry(id.to_string(), entry);

    VaultManager::save(&vault, &master_password, vault_file)?;
    println!("✓ Entry '{}' added successfully!", id);
    Ok(())
}

fn handle_get(id: &str, vault_file: Option<&str>) -> Result<(), Box<dyn Error>> {
    let master_password = read_password_secure("Enter master password: ")?;
    let vault = VaultManager::load(&master_password, vault_file)?;

    match vault.get_entry(id) {
        Some(entry) => {
            println!("\n--- {} ---", id);
            println!("Username: {}", entry.username);
            println!("Password: {}", entry.password);
            if let Some(note) = &entry.note {
                println!("Note: {}", note);
            }
            
            let copy_choice = read_line_optional("\nCopy password to clipboard? (y/N): ")?;
            if copy_choice.to_lowercase() == "y" || copy_choice.to_lowercase() == "yes" {
                copy_to_clipboard(&entry.password)?;
            }
        }
        None => {
            return Err(format!("Entry '{}' not found!", id).into());
        }
    }
    Ok(())
}

fn handle_list(vault_file: Option<&str>) -> Result<(), Box<dyn Error>> {
    let master_password = read_password_secure("Enter master password: ")?;
    let vault = VaultManager::load(&master_password, vault_file)?;

    if vault.is_empty() {
        println!("No entries found. Use 'passman add <id>' to add entries.");
        return Ok(());
    }

    println!("\nStored entries:");
    let mut entries: Vec<_> = vault.list_entries();
    entries.sort();
    
    for (i, id) in entries.iter().enumerate() {
        let entry = vault.get_entry(id).unwrap();
        println!("{}. {} ({})", i + 1, id, entry.username);
    }
    Ok(())
}

fn handle_remove(id: &str, vault_file: Option<&str>) -> Result<(), Box<dyn Error>> {
    let master_password = read_password_secure("Enter master password: ")?;
    let mut vault = VaultManager::load(&master_password, vault_file)?;

    match vault.remove_entry(id) {
        Some(_) => {
            VaultManager::save(&vault, &master_password, vault_file)?;
            println!("✓ Entry '{}' removed successfully!", id);
        }
        None => {
            return Err(format!("Entry '{}' not found!", id).into());
        }
    }
    Ok(())
}

fn handle_check(password: Option<&str>) -> Result<(), Box<dyn Error>> {
    let pwd = match password {
        Some(p) => Zeroizing::new(p.to_string()),
        None => read_password_secure("Enter password to analyze: ")?,
    };

    let (strength, suggestions) = analyze_password_strength(&pwd);
    
    println!("\nPassword Analysis:");
    println!("Strength: {}", strength);
    
    if !suggestions.is_empty() {
        println!("\nSuggestions for improvement:");
        for suggestion in suggestions {
            println!("  • {}", suggestion);
        }
    } else {
        println!("✓ This is a strong password!");
    }
    
    Ok(())
}

fn handle_vaults() -> Result<(), Box<dyn Error>> {
    use std::fs;
    
    println!("Available vault files:");
    
    let current_dir = std::env::current_dir()?;
    let entries = fs::read_dir(&current_dir)?;
    
    let mut vault_files = Vec::new();
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if let Some(file_name) = path.file_name() {
            let file_name_str = file_name.to_string_lossy();
            if file_name_str.ends_with(".dat") || file_name_str == "vault.dat" {
                vault_files.push(file_name_str.to_string());
            }
        }
    }
    
    if vault_files.is_empty() {
        println!("No vault files found in current directory.");
        println!("Use 'passman init' to create a new vault.");
    } else {
        vault_files.sort();
        for (i, file) in vault_files.iter().enumerate() {
            println!("{}. {}", i + 1, file);
        }
    }
    
    Ok(())
}

fn handle_edit(id: &str, vault_file: Option<&str>) -> Result<(), Box<dyn Error>> {
    let vault_file = vault_file.unwrap_or("vault.dat");
    
    if !std::path::Path::new(vault_file).exists() {
        return Err("Vault file does not exist. Run 'passman init' first.".into());
    }

    print!("Enter master password: ");
    std::io::stdout().flush()?;
    let master_password = Zeroizing::new(rpassword::read_password()?);

    let mut vault = VaultManager::load(&master_password, Some(vault_file))?;
    
    let entry = vault.get_entry(id).ok_or("Entry not found")?;
    
    println!("Editing entry: {}", id);
    println!("Current values (press Enter to keep current value):");
    
    // Get new username
    print!("Username [{}]: ", entry.username);
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let new_username = input.trim();
    let username = if new_username.is_empty() {
        entry.username.clone()
    } else {
        new_username.to_string()
    };
    
    // Get new password
    print!("Password (leave empty to keep current, type 'generate' to generate new): ");
    std::io::stdout().flush()?;
    let password_input = rpassword::read_password()?;
    let password = if password_input.is_empty() {
        entry.password.clone()
    } else if password_input == "generate" {
        let generated = generate_password(16);
        println!("Generated password: {}", generated);
        generated
    } else {
        password_input
    };
    
    // Get new note
    print!("Note [{}]: ", entry.note.as_deref().unwrap_or(""));
    std::io::stdout().flush()?;
    input.clear();
    std::io::stdin().read_line(&mut input)?;
    let new_note = input.trim();
    let note = if new_note.is_empty() {
        entry.note.clone()
    } else {
        Some(new_note.to_string())
    };
    
    // Create updated entry
    let updated_entry = Entry {
        username,
        password,
        note,
        created_at: entry.created_at,
        modified_at: chrono::Utc::now(),
        tags: entry.tags.clone(),
        url: entry.url.clone(),
        totp_secret: entry.totp_secret.clone(),
    };
    
    vault.add_entry(id.to_string(), updated_entry);
    VaultManager::save(&vault, &master_password, Some(vault_file))?;
    
    println!("✓ Entry '{}' updated successfully!", id);
    Ok(())
}

fn handle_generate(length: usize, _symbols: bool, memorable: bool) -> Result<(), Box<dyn Error>> {
    use crate::utils::{generate_password, generate_memorable_password, analyze_password_strength};
    
    let password = if memorable {
        generate_memorable_password(4) // Generate 4-word memorable password
    } else {
        generate_password(length)
    };
    
    println!("Generated password: {}", password);
    
    // Analyze strength
    let (strength, suggestions) = analyze_password_strength(&password);
    println!("Strength: {}", strength);
    
    if !suggestions.is_empty() {
        println!("Suggestions:");
        for suggestion in suggestions {
            println!("  • {}", suggestion);
        }
    }
    
    Ok(())
}

fn handle_transfer(transfer_cmd: TransferCommands, vault_file: Option<&str>) -> Result<(), Box<dyn Error>> {
    match transfer_cmd {
        TransferCommands::Export { output, format } => {
            let master_password = read_password_secure("Enter master password: ")?;
            let vault = VaultManager::load(&master_password, vault_file)?;            match format.as_str() {
                "json" => import_export::ImportExportManager::export_json(&vault, &output)?,
                "csv" => import_export::ImportExportManager::export_csv(&vault, &output)?,
                _ => return Err(format!("Unsupported export format: {}", format).into()),
            }
        }
        TransferCommands::Import { input, format, merge } => {
            let master_password = read_password_secure("Enter master password: ")?;
            
            // Create backup before import if vault exists
            if VaultManager::exists(vault_file) {
                if let Ok(backup_path) = import_export::ImportExportManager::create_auto_backup(vault_file) {
                    println!("📦 Created backup: {}", backup_path);
                }
            }            match format.as_str() {
                "json" => import_export::ImportExportManager::import_json(&input, &master_password, vault_file, merge)?,
                "csv" => import_export::ImportExportManager::import_csv(&input, &master_password, vault_file, merge)?,
                "chrome" | "firefox" => import_export::ImportExportManager::import_browser(&input, &master_password, vault_file, &format, merge)?,
                _ => return Err(format!("Unsupported import format: {}", format).into()),
            }
        }
    }
    Ok(())
}

fn handle_config(config_cmd: ConfigCommands) -> Result<(), Box<dyn Error>> {
    use crate::config::Config;
    
    match config_cmd {
        ConfigCommands::Show => {
            match Config::load() {
                Ok(config) => {
                    println!("Current configuration:");                    println!("  Default vault: {}", config.default_vault.display());
                    println!("  Password length: {}", config.default_password_length);
                    println!("  Auto lock timeout: {} minutes", config.auto_lock_timeout);
                    println!("  Clipboard clear timeout: {} seconds", config.clipboard_clear_timeout);
                }
                Err(e) => {
                    println!("Error loading config: {}", e);
                    println!("Using default configuration.");
                }
            }
        }        ConfigCommands::Set { key, value } => {
            let mut config = Config::load().unwrap_or_default();
            let value_clone = value.clone();
              match key.as_str() {
                "default_vault" => config.default_vault = value.into(),
                "default_password_length" => {
                    config.default_password_length = value.parse().map_err(|_| "Invalid number")?;
                }
                "auto_lock_timeout" => {
                    config.auto_lock_timeout = value.parse().map_err(|_| "Invalid number")?;
                }
                "clipboard_clear_timeout" => {
                    config.clipboard_clear_timeout = value.parse().map_err(|_| "Invalid number")?;
                }
                _ => return Err(format!("Unknown configuration key: {}", key).into()),
            }
            
            config.save()?;
            println!("✓ Configuration updated: {} = {}", key, value_clone);
        }
        ConfigCommands::Reset => {
            let config = Config::default();
            config.save()?;
            println!("✓ Configuration reset to defaults");
        }
    }
    Ok(())
}




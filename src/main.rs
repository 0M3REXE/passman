mod cli;
mod crypto;
mod vault;
mod model;
mod utils;
mod gui;
mod config;
mod health;
mod import_export;
mod secure_memory;
mod error;
mod backup;
mod logging;
mod two_factor;
mod tests;

use eframe::egui;
use cli::{Cli, Commands, TransferCommands, ConfigCommands};
use model::Entry;
use vault::VaultManager;
use utils::*;
use clap::Parser;
use std::error::Error;
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
        Commands::Init { description } => handle_init(vault_file, description),
        Commands::Add { id, username, password, note, url, generate, length } => {
            handle_add(&id, username, password, note, url, generate, length, vault_file)
        },
        Commands::Get { id, copy, show } => handle_get(&id, copy, show, vault_file),
        Commands::List { tag, search, verbose } => handle_list(tag, search, verbose, vault_file),
        Commands::Edit { id } => handle_edit(&id, vault_file),
        Commands::Remove { id, force } => handle_remove(&id, force, vault_file),
        Commands::Check { password, all } => handle_check(password.as_deref(), all),
        Commands::Vaults => handle_vaults(),
        Commands::Generate { length, symbols, no_ambiguous, memorable } => {
            handle_generate(length, symbols, no_ambiguous, memorable)
        },
        Commands::Transfer(transfer_cmd) => match transfer_cmd {
            TransferCommands::Export { output, format } => handle_export(&output, &format, vault_file),
            TransferCommands::Import { input, format, merge } => handle_import(&input, &format, merge, vault_file),
        },
        Commands::Config(config_cmd) => match config_cmd {
            ConfigCommands::Show => handle_config_show(),
            ConfigCommands::Set { key, value } => handle_config_set(&key, &value),
            ConfigCommands::Reset => handle_config_reset(),
        },
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn handle_init(vault_file: Option<&str>, description: Option<String>) -> Result<(), Box<dyn Error>> {
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
    if let Some(desc) = description {
        println!("Description: {}", desc);
    }
    Ok(())
}

fn handle_add(id: &str, username: Option<String>, password: Option<String>, note: Option<String>, url: Option<String>, generate: bool, length: usize, vault_file: Option<&str>) -> Result<(), Box<dyn Error>> {
    let master_password = read_password_secure("Enter master password: ")?;
    let mut vault = VaultManager::load(&master_password, vault_file)?;

    if vault.get_entry(id).is_some() {
        return Err(format!("Entry '{}' already exists!", id).into());
    }

    println!("Adding new entry for '{}'", id);
    
    // Get username - use provided or prompt
    let username = username.unwrap_or_else(|| {
        read_line("Username: ").unwrap_or_default()
    });
    
    // Handle password - generate or use provided or prompt
    let password = if generate {
        let generated = generate_password(length);
        println!("Generated password: {}", generated);
        let (strength, _) = analyze_password_strength(&generated);
        println!("Password strength: {}", strength);
        generated
    } else if let Some(provided_password) = password {
        let (strength, suggestions) = analyze_password_strength(&provided_password);
        println!("Password strength: {}", strength);
        if !suggestions.is_empty() {
            println!("Suggestions:");
            for suggestion in suggestions {
                println!("  • {}", suggestion);
            }
        }
        provided_password
    } else {
        let password_choice = read_line_optional("Generate password? (y/N): ")?;
        if password_choice.to_lowercase() == "y" || password_choice.to_lowercase() == "yes" {
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
        }
    };

    // Get note - use provided or prompt
    let note = note.or_else(|| {
        let note_input = read_line_optional("Note (optional): ").ok()?;
        if note_input.is_empty() { None } else { Some(note_input) }
    });

    let mut entry = Entry::new(username, password, note);
    if let Some(url_val) = url {
        entry.url = Some(url_val);
    }

    vault.add_entry(id.to_string(), entry);

    VaultManager::save(&vault, &master_password, vault_file)?;
    println!("✓ Entry '{}' added successfully!", id);
    Ok(())
}

fn handle_get(id: &str, copy: bool, show: bool, vault_file: Option<&str>) -> Result<(), Box<dyn Error>> {
    let master_password = read_password_secure("Enter master password: ")?;
    let vault = VaultManager::load(&master_password, vault_file)?;

    match vault.get_entry(id) {
        Some(entry) => {
            println!("\n--- {} ---", id);
            println!("Username: {}", entry.username);
            
            if show {
                println!("Password: {}", entry.password);
            } else {
                println!("Password: [hidden - use --show to display]");
            }
            
            if let Some(note) = &entry.note {
                println!("Note: {}", note);
            }
            if let Some(url) = &entry.url {
                println!("URL: {}", url);
            }
            
            if copy {
                copy_to_clipboard(&entry.password)?;
                println!("✓ Password copied to clipboard!");
            } else if !show {
                let copy_choice = read_line_optional("\nCopy password to clipboard? (y/N): ")?;
                if copy_choice.to_lowercase() == "y" || copy_choice.to_lowercase() == "yes" {
                    copy_to_clipboard(&entry.password)?;
                    println!("✓ Password copied to clipboard!");
                }
            }
        }
        None => {
            return Err(format!("Entry '{}' not found!", id).into());
        }
    }
    Ok(())
}

fn handle_list(tag: Option<String>, search: Option<String>, verbose: bool, vault_file: Option<&str>) -> Result<(), Box<dyn Error>> {
    let master_password = read_password_secure("Enter master password: ")?;
    let vault = VaultManager::load(&master_password, vault_file)?;

    if vault.is_empty() {
        println!("No entries found. Use 'passman add <id>' to add entries.");
        return Ok(());
    }

    println!("\nStored entries:");
    let mut entries: Vec<_> = vault.list_entries();
    entries.sort();
    
    // Filter by search pattern if provided
    if let Some(search_term) = &search {
        entries.retain(|id| {
            if let Some(entry) = vault.get_entry(id) {
                id.contains(search_term) || 
                entry.username.contains(search_term) ||
                entry.note.as_ref().map_or(false, |n| n.contains(search_term))
            } else {
                false
            }
        });
    }
    
    // Filter by tag if provided (placeholder - tags not fully implemented yet)
    if let Some(_tag_filter) = &tag {
        println!("Note: Tag filtering not yet implemented");
    }
    
    for (i, id) in entries.iter().enumerate() {
        let entry = vault.get_entry(id).unwrap();
        if verbose {
            println!("{}. {} ({})", i + 1, id, entry.username);
            if let Some(note) = &entry.note {
                println!("   Note: {}", note);
            }
            if let Some(url) = &entry.url {
                println!("   URL: {}", url);
            }
        } else {
            println!("{}. {} ({})", i + 1, id, entry.username);
        }
    }
    
    if entries.is_empty() && search.is_some() {
        println!("No entries found matching search criteria.");
    }
    
    Ok(())
}

fn handle_remove(id: &str, force: bool, vault_file: Option<&str>) -> Result<(), Box<dyn Error>> {
    let master_password = read_password_secure("Enter master password: ")?;
    let mut vault = VaultManager::load(&master_password, vault_file)?;

    // Check if entry exists
    if vault.get_entry(id).is_none() {
        return Err(format!("Entry '{}' not found!", id).into());
    }

    // Confirm removal unless force flag is used
    if !force {
        let confirm = read_line_optional(&format!("Remove entry '{}'? (y/N): ", id))?;
        if confirm.to_lowercase() != "y" && confirm.to_lowercase() != "yes" {
            println!("Operation cancelled.");
            return Ok(());
        }
    }

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

fn handle_check(password: Option<&str>, all: bool) -> Result<(), Box<dyn Error>> {
    if all {
        println!("Checking all passwords in vault...");
        // This would require vault access - for now, just show placeholder
        println!("Note: Vault-wide password checking not yet implemented");
        return Ok(());
    }

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
    let master_password = read_password_secure("Enter master password: ")?;
    let mut vault = VaultManager::load(&master_password, vault_file)?;

    match vault.get_entry(id) {
        Some(existing_entry) => {
            println!("Editing entry: {}", id);
            println!("Current username: {}", existing_entry.username);
            
            let new_username = read_line_optional(&format!("New username (current: {}): ", existing_entry.username))?;
            let username = if new_username.trim().is_empty() {
                existing_entry.username.clone()
            } else {
                new_username
            };
            
            let password_choice = read_line_optional("Update password? (y/N): ")?;
            let password = if password_choice.to_lowercase() == "y" || password_choice.to_lowercase() == "yes" {
                let gen_choice = read_line_optional("Generate new password? (y/N): ")?;
                if gen_choice.to_lowercase() == "y" || gen_choice.to_lowercase() == "yes" {
                    let generated = generate_password(16);
                    println!("Generated password: {}", generated);
                    generated
                } else {
                    read_password_secure("New password: ")?.to_string()
                }
            } else {
                existing_entry.password.clone()
            };
            
            let current_note = existing_entry.note.as_deref().unwrap_or("");
            let new_note = read_line_optional(&format!("Note (current: {}): ", current_note))?;
            let note = if new_note.trim().is_empty() && !current_note.is_empty() {
                existing_entry.note.clone()
            } else if new_note.trim().is_empty() {
                None
            } else {
                Some(new_note)
            };
            
            let mut updated_entry = Entry::new(username, password, note);
            updated_entry.url = existing_entry.url.clone();
            updated_entry.tags = existing_entry.tags.clone();
            updated_entry.totp_secret = existing_entry.totp_secret.clone();
            
            vault.add_entry(id.to_string(), updated_entry);
            VaultManager::save(&vault, &master_password, vault_file)?;
            println!("✓ Entry '{}' updated successfully!", id);
        }
        None => {
            return Err(format!("Entry '{}' not found!", id).into());
        }
    }
    Ok(())
}

fn handle_generate(length: usize, symbols: bool, no_ambiguous: bool, memorable: bool) -> Result<(), Box<dyn Error>> {
    if memorable {
        println!("Note: Memorable password generation not yet implemented, generating regular password");
    }
    
    let password = if symbols && !no_ambiguous {
        generate_password(length)
    } else {
        // For now, use basic generation - could be enhanced
        generate_password(length)
    };
    
    println!("Generated password: {}", password);
    
    let (strength, suggestions) = analyze_password_strength(&password);
    println!("Password strength: {}", strength);
    
    if !suggestions.is_empty() {
        println!("Suggestions:");
        for suggestion in suggestions {
            println!("  • {}", suggestion);
        }
    }
    
    let copy_choice = read_line_optional("Copy to clipboard? (y/N): ")?;
    if copy_choice.to_lowercase() == "y" || copy_choice.to_lowercase() == "yes" {
        copy_to_clipboard(&password)?;
        println!("✓ Password copied to clipboard!");
    }
    
    Ok(())
}

fn handle_export(output: &str, format: &str, vault_file: Option<&str>) -> Result<(), Box<dyn Error>> {
    let master_password = read_password_secure("Enter master password: ")?;
    let vault = VaultManager::load(&master_password, vault_file)?;
    
    println!("Exporting vault to {} in {} format...", output, format);
    
    match format.to_lowercase().as_str() {
        "json" => {
            use crate::import_export::ImportExportManager;
            ImportExportManager::export_json(&vault, output)?;
            println!("✓ Vault exported to JSON successfully!");
        }
        "csv" => {
            use crate::import_export::ImportExportManager;
            ImportExportManager::export_csv(&vault, output)?;
            println!("✓ Vault exported to CSV successfully!");
        }
        _ => {
            return Err(format!("Unsupported export format: {}", format).into());
        }
    }
    
    Ok(())
}

fn handle_import(input: &str, format: &str, merge: bool, vault_file: Option<&str>) -> Result<(), Box<dyn Error>> {
    let master_password = read_password_secure("Enter master password: ")?;
    
    println!("Importing from {} in {} format...", input, format);
    if merge {
        println!("Merging with existing vault data");
    } else {
        println!("Warning: This will overwrite existing vault data");
        let confirm = read_line_optional("Continue? (y/N): ")?;
        if confirm.to_lowercase() != "y" && confirm.to_lowercase() != "yes" {
            println!("Import cancelled.");
            return Ok(());
        }
    }
    
    match format.to_lowercase().as_str() {
        "json" => {
            use crate::import_export::ImportExportManager;
            ImportExportManager::import_json(input, &master_password, vault_file, merge)?;
            println!("✓ JSON data imported successfully!");
        }
        "csv" => {
            use crate::import_export::ImportExportManager;
            ImportExportManager::import_csv(input, &master_password, vault_file, merge)?;
            println!("✓ CSV data imported successfully!");
        }        "chrome" => {
            use crate::import_export::ImportExportManager;
            ImportExportManager::import_browser(input, &master_password, vault_file, "chrome", merge)?;
            println!("✓ Chrome data imported successfully!");
        }
        _ => {
            return Err(format!("Unsupported import format: {}", format).into());
        }
    }
    
    Ok(())
}

fn handle_config_show() -> Result<(), Box<dyn Error>> {
    println!("Configuration settings:");
    println!("Note: Configuration management not yet fully implemented");
    println!("- Default vault file: vault.dat");
    println!("- Password generation length: 16");
    println!("- Security features: Memory locking enabled");
    Ok(())
}

fn handle_config_set(key: &str, value: &str) -> Result<(), Box<dyn Error>> {
    println!("Setting config: {} = {}", key, value);
    println!("Note: Configuration management not yet fully implemented");
    Ok(())
}

fn handle_config_reset() -> Result<(), Box<dyn Error>> {
    println!("Resetting configuration to defaults...");
    println!("Note: Configuration management not yet fully implemented");
    Ok(())
}




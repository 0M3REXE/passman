mod cli;
mod crypto;
mod vault;
mod model;
mod utils;
mod gui;
mod health;
mod import_export;
mod secure_clipboard;
mod session;
mod error;
mod config;

use eframe::egui;
use cli::{Cli, Commands, TransferCommands, ConfigCommands};
use model::Entry;
use vault::VaultManager;
use utils::*;
use clap::Parser;
use std::error::Error;
use zeroize::Zeroizing;

// Re-export commonly used types
pub use error::{PassmanError, PassmanResult};
pub use config::Config;

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
        Commands::Init { description: _ } => handle_init(vault_file),
        Commands::Add { id, .. } => handle_add(&id, vault_file),
        Commands::Get { id, copy, show } => handle_get(&id, vault_file, copy, show),
        Commands::List { search, verbose, .. } => handle_list(vault_file, search.as_deref(), verbose),
        Commands::Edit { id } => handle_edit(&id, vault_file),
        Commands::Remove { id, force } => handle_remove(&id, vault_file, force),
        Commands::Check { password, all } => handle_check(password.as_deref(), all, vault_file),
        Commands::Vaults => handle_vaults(),
        Commands::Generate { length, symbols, no_ambiguous, memorable } => {
            handle_generate(length, symbols, no_ambiguous, memorable)
        },
        Commands::Transfer(transfer_cmd) => handle_transfer(transfer_cmd, vault_file),
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

fn handle_get(id: &str, vault_file: Option<&str>, copy: bool, show: bool) -> Result<(), Box<dyn Error>> {
    let master_password = read_password_secure("Enter master password: ")?;
    let vault = VaultManager::load(&master_password, vault_file)?;

    match vault.get_entry(id) {
        Some(entry) => {
            println!("\n--- {} ---", id);
            println!("Username: {}", entry.username);
            
            if show {
                println!("Password: {}", entry.password);
            } else {
                println!("Password: {}", "*".repeat(entry.password.len().min(16)));
            }
            
            if let Some(note) = &entry.note {
                println!("Note: {}", note);
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

fn handle_list(vault_file: Option<&str>, search: Option<&str>, verbose: bool) -> Result<(), Box<dyn Error>> {
    let master_password = read_password_secure("Enter master password: ")?;
    let vault = VaultManager::load(&master_password, vault_file)?;

    if vault.is_empty() {
        println!("No entries found. Use 'passman add <id>' to add entries.");
        return Ok(());
    }

    let mut entries: Vec<_> = vault.list_entries();
    entries.sort();
    
    // Filter by search term if provided
    let filtered_entries: Vec<_> = if let Some(pattern) = search {
        let pattern_lower = pattern.to_lowercase();
        entries.into_iter()
            .filter(|id| {
                let id_lower = id.to_lowercase();
                if id_lower.contains(&pattern_lower) {
                    return true;
                }
                if let Some(entry) = vault.get_entry(id) {
                    if entry.username.to_lowercase().contains(&pattern_lower) {
                        return true;
                    }
                    if let Some(note) = &entry.note {
                        if note.to_lowercase().contains(&pattern_lower) {
                            return true;
                        }
                    }
                }
                false
            })
            .collect()
    } else {
        entries
    };

    if filtered_entries.is_empty() {
        println!("No entries match your search criteria.");
        return Ok(());
    }

    println!("\nStored entries ({} found):", filtered_entries.len());
    println!("{}", "-".repeat(50));
    
    for (i, id) in filtered_entries.iter().enumerate() {
        let entry = vault.get_entry(id).unwrap();
        if verbose {
            println!("{}. {}", i + 1, id);
            println!("   Username: {}", entry.username);
            println!("   Password: {}", "*".repeat(entry.password.len().min(12)));
            if let Some(note) = &entry.note {
                println!("   Note: {}", note);
            }
            let (strength, _) = analyze_password_strength(&entry.password);
            println!("   Strength: {}", strength);
            println!();
        } else {
            println!("{}. {} ({})", i + 1, id, entry.username);
        }
    }
    Ok(())
}

fn handle_remove(id: &str, vault_file: Option<&str>, force: bool) -> Result<(), Box<dyn Error>> {
    let master_password = read_password_secure("Enter master password: ")?;
    let mut vault = VaultManager::load(&master_password, vault_file)?;

    if vault.get_entry(id).is_none() {
        return Err(format!("Entry '{}' not found!", id).into());
    }

    // Confirm deletion unless force flag is set
    if !force {
        let confirm = read_line_optional(&format!("Are you sure you want to delete '{}'? (y/N): ", id))?;
        if confirm.to_lowercase() != "y" && confirm.to_lowercase() != "yes" {
            println!("Deletion cancelled.");
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

fn handle_check(password: Option<&str>, all: bool, vault_file: Option<&str>) -> Result<(), Box<dyn Error>> {
    if all {
        // Check all passwords in vault
        let master_password = read_password_secure("Enter master password: ")?;
        let vault = VaultManager::load(&master_password, vault_file)?;
        
        if vault.is_empty() {
            println!("No entries in vault.");
            return Ok(());
        }

        println!("\nPassword Strength Analysis:");
        println!("{}", "=".repeat(60));
        
        let mut weak_count = 0;
        let mut entries: Vec<_> = vault.list_entries();
        entries.sort();

        for id in entries {
            let entry = vault.get_entry(&id).unwrap();
            let (strength, suggestions) = analyze_password_strength(&entry.password);
            
            let status_icon = if suggestions.is_empty() { "✓" } else { "⚠" };
            println!("{} {} - {}", status_icon, id, strength);
            
            if !suggestions.is_empty() {
                weak_count += 1;
                for suggestion in &suggestions {
                    println!("    • {}", suggestion);
                }
            }
        }
        
        println!("{}", "=".repeat(60));
        if weak_count > 0 {
            println!("⚠ {} password(s) need improvement", weak_count);
        } else {
            println!("✓ All passwords are strong!");
        }
    } else {
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

    let entry = match vault.get_entry(id) {
        Some(e) => e.clone(),
        None => return Err(format!("Entry '{}' not found!", id).into()),
    };

    println!("\nEditing entry '{}' (press Enter to keep current value)", id);
    println!("{}", "-".repeat(50));

    // Edit username
    println!("Current username: {}", entry.username);
    let new_username = read_line_optional("New username: ")?;
    let username = if new_username.is_empty() { entry.username.clone() } else { new_username };

    // Edit password
    println!("Current password: {}", "*".repeat(entry.password.len().min(16)));
    let password_choice = read_line_optional("Change password? (y/N/g for generate): ")?;
    let password = match password_choice.to_lowercase().as_str() {
        "y" | "yes" => {
            let pwd = read_password_secure("New password: ")?;
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
        "g" | "gen" | "generate" => {
            let len_str = read_line_optional("Password length (default 16): ")?;
            let len: usize = len_str.parse().unwrap_or(16);
            let generated = generate_password(len);
            println!("Generated password: {}", generated);
            let (strength, _) = analyze_password_strength(&generated);
            println!("Password strength: {}", strength);
            generated
        }
        _ => entry.password.clone(),
    };

    // Edit note
    let current_note = entry.note.clone().unwrap_or_default();
    if !current_note.is_empty() {
        println!("Current note: {}", current_note);
    }
    let new_note = read_line_optional("New note (or '-' to remove): ")?;
    let note = match new_note.as_str() {
        "" => entry.note.clone(),
        "-" => None,
        _ => Some(new_note),
    };

    // Create updated entry and save (add_entry with insert replaces existing)
    let updated_entry = Entry::new(username, password, note);
    vault.add_entry(id.to_string(), updated_entry);
    VaultManager::save(&vault, &master_password, vault_file)?;

    println!("\n✓ Entry '{}' updated successfully!", id);
    Ok(())
}

fn handle_generate(length: usize, symbols: bool, no_ambiguous: bool, memorable: bool) -> Result<(), Box<dyn Error>> {
    let password = if memorable {
        generate_memorable_password(4)
    } else {
        generate_password_with_options(length, symbols, !no_ambiguous)
    };

    println!("\nGenerated Password: {}", password);
    
    let (strength, suggestions) = analyze_password_strength(&password);
    println!("Strength: {}", strength);
    
    if !suggestions.is_empty() {
        println!("Note:");
        for suggestion in suggestions {
            println!("  • {}", suggestion);
        }
    }

    let copy_choice = read_line_optional("\nCopy to clipboard? (y/N): ")?;
    if copy_choice.to_lowercase() == "y" || copy_choice.to_lowercase() == "yes" {
        copy_to_clipboard(&password)?;
        println!("✓ Password copied to clipboard!");
    }

    Ok(())
}

fn generate_password_with_options(length: usize, include_symbols: bool, include_ambiguous: bool) -> String {
    use rand::seq::SliceRandom;
    
    let mut charset: Vec<char> = Vec::new();
    
    // Lowercase
    charset.extend('a'..='z');
    // Uppercase
    charset.extend('A'..='Z');
    // Digits
    charset.extend('0'..='9');
    
    if include_symbols {
        charset.extend("!@#$%^&*()_+-=[]{}|;:,.<>?".chars());
    }
    
    if !include_ambiguous {
        // Remove ambiguous characters
        let ambiguous = ['0', 'O', 'o', '1', 'l', 'I', '|'];
        charset.retain(|c| !ambiguous.contains(c));
    }
    
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| *charset.choose(&mut rng).unwrap())
        .collect()
}

fn generate_memorable_password(word_count: usize) -> String {
    use rand::seq::SliceRandom;
    use rand::Rng;
    
    let words = vec![
        "apple", "banana", "cherry", "dragon", "eagle", "falcon", "garden", "harbor",
        "island", "jungle", "knight", "lemon", "mountain", "nebula", "ocean", "phoenix",
        "quartz", "river", "sunset", "thunder", "umbrella", "valley", "winter", "xenon",
        "yellow", "zenith", "anchor", "bridge", "castle", "diamond", "empire", "forest",
        "glacier", "horizon", "ivory", "jasmine", "kingdom", "lantern", "marble", "neptune",
        "orchid", "palace", "quantum", "rainbow", "silver", "tornado", "universe", "volcano",
    ];
    
    let mut rng = rand::thread_rng();
    let mut result = Vec::new();
    
    for _ in 0..word_count {
        let word = words.choose(&mut rng).unwrap();
        // Capitalize first letter
        let capitalized: String = word.chars().enumerate()
            .map(|(i, c)| if i == 0 { c.to_uppercase().next().unwrap() } else { c })
            .collect();
        result.push(capitalized);
    }
    
    // Add a random number
    let num: u16 = rng.gen_range(10..100);
    result.push(num.to_string());
    
    // Add a random symbol
    let symbols = ['!', '@', '#', '$', '%', '&', '*'];
    let symbol = symbols.choose(&mut rng).unwrap();
    
    result.join("-") + &symbol.to_string()
}

fn handle_transfer(cmd: TransferCommands, vault_file: Option<&str>) -> Result<(), Box<dyn Error>> {
    use import_export::ImportExportManager;
    
    match cmd {
        TransferCommands::Export { output, format } => {
            let master_password = read_password_secure("Enter master password: ")?;
            let vault = VaultManager::load(&master_password, vault_file)?;
            
            match format.to_lowercase().as_str() {
                "json" => {
                    ImportExportManager::export_json(&vault, &output)?;
                }
                "csv" => {
                    ImportExportManager::export_csv(&vault, &output)?;
                }
                _ => return Err(format!("Unsupported export format: {}. Use 'json' or 'csv'.", format).into()),
            }
            
            println!("✓ Vault exported to '{}' successfully!", output);
            println!("⚠ Warning: Exported file contains unencrypted passwords. Handle with care!");
        }
        TransferCommands::Import { input, format, merge } => {
            let master_password = read_password_secure("Enter master password: ")?;
            
            // The import functions handle vault creation/loading internally
            match format.to_lowercase().as_str() {
                "json" => {
                    ImportExportManager::import_json(&input, &master_password, vault_file, merge)?;
                }
                "csv" => {
                    ImportExportManager::import_csv(&input, &master_password, vault_file, merge)?;
                }
                "chrome" | "firefox" => {
                    ImportExportManager::import_browser(&input, &master_password, vault_file, &format.to_lowercase(), merge)?;
                }
                _ => return Err(format!("Unsupported import format: {}. Use 'json', 'csv', 'chrome', or 'firefox'.", format).into()),
            }
        }
    }
    
    Ok(())
}

fn handle_config(cmd: ConfigCommands) -> Result<(), Box<dyn Error>> {
    use config::{get_config, get_config_mut, save_config, reload_config, Config as AppConfig};
    
    match cmd {
        ConfigCommands::Show => {
            let config = get_config();
            println!("\nCurrent Configuration:");
            println!("{}", "=".repeat(50));
            
            println!("\n[General]");
            println!("  default_vault: {}", config.general.default_vault);
            
            println!("\n[Security]");
            println!("  lock_timeout_secs: {} ({})", 
                config.security.lock_timeout_secs,
                format_duration(config.security.lock_timeout_secs));
            println!("  clipboard_timeout_secs: {}", config.security.clipboard_timeout_secs);
            println!("  clear_clipboard_on_lock: {}", config.security.clear_clipboard_on_lock);
            println!("  max_failed_attempts: {}", config.security.max_failed_attempts);
            println!("  min_password_length: {}", config.security.min_password_length);
            
            println!("\n[Password Generation]");
            println!("  default_length: {}", config.password.default_length);
            println!("  include_uppercase: {}", config.password.include_uppercase);
            println!("  include_lowercase: {}", config.password.include_lowercase);
            println!("  include_numbers: {}", config.password.include_numbers);
            println!("  include_symbols: {}", config.password.include_symbols);
            println!("  exclude_ambiguous: {}", config.password.exclude_ambiguous);
            
            println!("\n[UI]");
            println!("  theme: {}", config.ui.theme);
            println!("  show_password_strength: {}", config.ui.show_password_strength);
            
            println!("\n[Backup]");
            println!("  auto_backup: {}", config.backup.auto_backup);
            println!("  max_backups: {}", config.backup.max_backups);
        }
        ConfigCommands::Set { key, value } => {
            let mut config = get_config_mut();
            
            match key.to_lowercase().as_str() {
                "security.lock_timeout_secs" | "lock_timeout" => {
                    config.security.lock_timeout_secs = value.parse()
                        .map_err(|_| format!("Invalid number: {}", value))?;
                }
                "security.clipboard_timeout_secs" | "clipboard_timeout" => {
                    config.security.clipboard_timeout_secs = value.parse()
                        .map_err(|_| format!("Invalid number: {}", value))?;
                }
                "security.max_failed_attempts" | "max_attempts" => {
                    config.security.max_failed_attempts = value.parse()
                        .map_err(|_| format!("Invalid number: {}", value))?;
                }
                "security.min_password_length" | "min_password" => {
                    config.security.min_password_length = value.parse()
                        .map_err(|_| format!("Invalid number: {}", value))?;
                }
                "password.default_length" | "password_length" => {
                    config.password.default_length = value.parse()
                        .map_err(|_| format!("Invalid number: {}", value))?;
                }
                "password.include_symbols" | "symbols" => {
                    config.password.include_symbols = value.parse()
                        .map_err(|_| format!("Invalid boolean: {}", value))?;
                }
                "password.exclude_ambiguous" | "exclude_ambiguous" => {
                    config.password.exclude_ambiguous = value.parse()
                        .map_err(|_| format!("Invalid boolean: {}", value))?;
                }
                "ui.theme" | "theme" => {
                    config.ui.theme = value.clone();
                }
                "backup.auto_backup" | "auto_backup" => {
                    config.backup.auto_backup = value.parse()
                        .map_err(|_| format!("Invalid boolean: {}", value))?;
                }
                "backup.max_backups" | "max_backups" => {
                    config.backup.max_backups = value.parse()
                        .map_err(|_| format!("Invalid number: {}", value))?;
                }
                "general.default_vault" | "default_vault" => {
                    config.general.default_vault = value.clone();
                }
                _ => {
                    return Err(format!("Unknown configuration key: {}", key).into());
                }
            }
            
            drop(config); // Release write lock before saving
            save_config()?;
            println!("✓ Configuration updated: {} = {}", key, value);
        }
        ConfigCommands::Reset => {
            let confirm = read_line_optional("Reset all configuration to defaults? (y/N): ")?;
            if confirm.to_lowercase() == "y" || confirm.to_lowercase() == "yes" {
                // Delete config file to reset
                let config_path = AppConfig::config_path();
                if config_path.exists() {
                    std::fs::remove_file(&config_path)?;
                }
                reload_config();
                println!("✓ Configuration reset to defaults!");
            } else {
                println!("Reset cancelled.");
            }
        }
    }
    
    Ok(())
}

fn format_duration(secs: u64) -> String {
    if secs == 0 {
        "disabled".to_string()
    } else if secs < 60 {
        format!("{} seconds", secs)
    } else if secs < 3600 {
        format!("{} minutes", secs / 60)
    } else {
        format!("{} hours", secs / 3600)
    }
}




mod cli;
mod crypto;
mod vault;
mod model;
mod utils;
mod gui;

use eframe::egui;
use cli::{Cli, Commands};
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
    let vault_file = cli.vault.as_deref();

    let result = match cli.command {
        Commands::Init => handle_init(vault_file),
        Commands::Add { id } => handle_add(&id, vault_file),
        Commands::Get { id } => handle_get(&id, vault_file),
        Commands::List => handle_list(vault_file),
        Commands::Remove { id } => handle_remove(&id, vault_file),
        Commands::Check { password } => handle_check(password.as_deref()),
        Commands::Vaults => handle_vaults(),
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




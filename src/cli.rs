use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "passman", version = "1.0", author = "0m3rexe")]
#[command(about = "A rapid fast password manager built with rust", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    /// Specify vault file (default: vault.dat)
    #[arg(short, long, global = true)]
    pub vault: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Set up master password, create encrypted vault
    Init,
    /// Add new entry (interactive)
    Add { id: String },
    /// Print or copy credentials
    Get { id: String },
    /// List all saved entries
    List,    /// Remove an entry
    #[command(name = "rm")]
    Remove { id: String },
    /// Analyze password strength
    Check { password: Option<String> },
    /// List available vaults
    Vaults,
}

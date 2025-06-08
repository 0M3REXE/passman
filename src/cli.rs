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
    
    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
    
    /// Quiet mode (minimal output)
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Set up master password, create encrypted vault
    Init {
        /// Description for the vault
        #[arg(short, long)]
        description: Option<String>,
    },
    
    /// Add new entry (interactive)
    Add { 
        id: String,
        /// Username/email
        #[arg(short, long)]
        username: Option<String>,
        /// Password (if not provided, will be generated or prompted)
        #[arg(short, long)]
        password: Option<String>,
        /// Note/description
        #[arg(short, long)]
        note: Option<String>,
        /// URL associated with this entry
        #[arg(long)]
        url: Option<String>,
        /// Generate password automatically
        #[arg(short, long)]
        generate: bool,
        /// Password length for generation
        #[arg(short, long, default_value = "16")]
        length: usize,
    },
    
    /// Print or copy credentials
    Get { 
        id: String,
        /// Copy password to clipboard instead of displaying
        #[arg(short, long)]
        copy: bool,
        /// Show password in plaintext
        #[arg(short, long)]
        show: bool,
    },
    
    /// List all saved entries
    List {
        /// Filter entries by tag
        #[arg(short, long)]
        tag: Option<String>,
        /// Search entries by pattern
        #[arg(short, long)]
        search: Option<String>,
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Edit an existing entry
    Edit { id: String },
    
    /// Remove an entry
    #[command(name = "rm")]
    Remove { 
        id: String,
        /// Force removal without confirmation
        #[arg(short, long)]
        force: bool,
    },
    
    /// Analyze password strength
    Check { 
        password: Option<String>,
        /// Check all passwords in vault
        #[arg(short, long)]
        all: bool,
    },
    
    /// List available vaults
    Vaults,
    
    /// Generate password
    Generate {
        /// Password length
        #[arg(short, long, default_value = "16")]
        length: usize,
        /// Include symbols
        #[arg(long)]
        symbols: bool,
        /// Exclude ambiguous characters
        #[arg(long)]
        no_ambiguous: bool,
        /// Generate memorable password
        #[arg(short, long)]
        memorable: bool,
    },
    
    /// Import/Export operations
    #[command(subcommand)]
    Transfer(TransferCommands),
    
    /// Configuration management
    #[command(subcommand)]
    Config(ConfigCommands),
}

#[derive(Subcommand)]
pub enum TransferCommands {
    /// Export vault to various formats
    Export {
        /// Output file path
        #[arg(short, long)]
        output: String,
        /// Export format (json, csv)
        #[arg(short, long, default_value = "json")]
        format: String,
    },
      /// Import from various formats
    Import {
        /// Input file path
        #[arg(short, long)]
        input: String,
        /// Import format (json, csv, chrome, firefox)
        #[arg(short, long)]
        format: String,
        /// Merge with existing vault instead of overwriting
        #[arg(short, long)]
        merge: bool,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,
    
    /// Set configuration value
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },
    
    /// Reset to default configuration
    Reset,
}

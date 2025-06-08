use std::io::{self, Write};
use std::fs::File;
use std::path::Path;
use clipboard::{ClipboardProvider, ClipboardContext};
use regex::Regex;
use zeroize::Zeroizing;
use crate::config::PasswordConfig;

/// Copy text to clipboard with proper error handling
pub fn copy_to_clipboard(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut ctx: ClipboardContext = ClipboardProvider::new()?;
    ctx.set_contents(text.to_owned())?;
    println!("✓ Copied to clipboard");
    Ok(())
}

/// Read password securely from stdin
pub fn read_password_secure(prompt: &str) -> Result<Zeroizing<String>, Box<dyn std::error::Error>> {
    print!("{}", prompt);
    io::stdout().flush()?;
    
    // Check if stdin is from a terminal (interactive) or piped
    let password = if atty::is(atty::Stream::Stdin) {
        // Interactive mode - use secure password reading
        Zeroizing::new(rpassword::read_password()?)
    } else {
        // Non-interactive mode (piped input) - read normally
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Zeroizing::new(input.trim().to_string())
    };
    
    if password.trim().is_empty() {
        return Err("Password cannot be empty".into());
    }
    Ok(password)
}

/// Read line from stdin with validation
pub fn read_line(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    loop {
        print!("{}", prompt);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let trimmed = input.trim().to_string();
        if !trimmed.is_empty() {
            return Ok(trimmed);
        }
        println!("Input cannot be empty. Please try again.");
    }
}

/// Check if file exists
#[allow(dead_code)]
pub fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

/// Create file if it doesn't exist
#[allow(dead_code)]
pub fn ensure_file_exists(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !file_exists(path) {
        File::create(path)?;
    }
    Ok(())
}

/// Read entire file as bytes
#[allow(dead_code)]
pub fn read_file_bytes(path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    Ok(std::fs::read(path)?)
}

/// Write bytes to file
#[allow(dead_code)]
pub fn write_file_bytes(path: &str, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write(path, data)?;
    Ok(())
}

/// Read line from stdin with optional input
pub fn read_line_optional(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}
pub fn generate_password(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789\
                            !@#$%^&*()_+-=[]{}|;:,.<>?";
    
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Password strength levels
#[derive(Debug, PartialEq, Clone)]
pub enum PasswordStrength {
    VeryWeak,
    Weak,
    Fair,
    Good,
    Strong,
}

impl std::fmt::Display for PasswordStrength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PasswordStrength::VeryWeak => write!(f, "Very Weak"),
            PasswordStrength::Weak => write!(f, "Weak"),
            PasswordStrength::Fair => write!(f, "Fair"),
            PasswordStrength::Good => write!(f, "Good"),
            PasswordStrength::Strong => write!(f, "Strong"),
        }
    }
}

/// Analyze password strength
pub fn analyze_password_strength(password: &str) -> (PasswordStrength, Vec<String>) {
    let mut score = 0;
    let mut suggestions = Vec::new();
    
    // Length check
    if password.len() >= 8 {
        score += 1;
    } else {
        suggestions.push("Use at least 8 characters".to_string());
    }
    
    if password.len() >= 12 {
        score += 1;
    } else if password.len() >= 8 {
        suggestions.push("Consider using 12+ characters for better security".to_string());
    }
    
    // Character type checks
    let has_lowercase = Regex::new(r"[a-z]").unwrap().is_match(password);
    let has_uppercase = Regex::new(r"[A-Z]").unwrap().is_match(password);
    let has_numbers = Regex::new(r"\d").unwrap().is_match(password);
    let has_symbols = Regex::new(r"[!@#$%^&*()_+\-=\[\]{}|;:,.<>?]").unwrap().is_match(password);
    
    if has_lowercase { score += 1; } else { suggestions.push("Add lowercase letters".to_string()); }
    if has_uppercase { score += 1; } else { suggestions.push("Add uppercase letters".to_string()); }
    if has_numbers { score += 1; } else { suggestions.push("Add numbers".to_string()); }
    if has_symbols { score += 1; } else { suggestions.push("Add special characters".to_string()); }
      // Check for repeated characters (simple approach)
    let mut has_repeated = false;
    let chars: Vec<char> = password.chars().collect();
    for i in 0..chars.len().saturating_sub(2) {
        if chars[i] == chars[i + 1] && chars[i + 1] == chars[i + 2] {
            has_repeated = true;
            break;
        }
    }
    
    if has_repeated {
        score -= 1;
        suggestions.push("Avoid repeating characters".to_string());
    }
    
    if Regex::new(r"(012|123|234|345|456|567|678|789|890|abc|bcd|cde|def|efg|fgh|ghi|hij|ijk|jkl|klm|lmn|mno|nop|opq|pqr|qrs|rst|stu|tuv|uvw|vwx|wxy|xyz)").unwrap().is_match(&password.to_lowercase()) {
        score -= 1;
        suggestions.push("Avoid sequential characters".to_string());
    }
    
    // Common passwords check
    let common_passwords = ["password", "123456", "password123", "admin", "qwerty", "letmein"];
    if common_passwords.iter().any(|&p| password.to_lowercase().contains(p)) {
        score -= 2;
        suggestions.push("Avoid common passwords".to_string());
    }
    
    let strength = match score {
        s if s <= 1 => PasswordStrength::VeryWeak,
        2 => PasswordStrength::Weak,
        3 => PasswordStrength::Fair,
        4 => PasswordStrength::Good,
        _ => PasswordStrength::Strong,
    };
    
    (strength, suggestions)
}

#[allow(dead_code)]
pub fn generate_password_with_config(length: usize, config: &PasswordConfig) -> String {
    use rand::Rng;
    
    let mut charset = Vec::new();
    
    if config.include_lowercase {
        charset.extend_from_slice(b"abcdefghijklmnopqrstuvwxyz");
    }
    if config.include_uppercase {
        charset.extend_from_slice(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }
    if config.include_numbers {
        charset.extend_from_slice(b"0123456789");
    }
    if config.include_symbols {
        charset.extend_from_slice(b"!@#$%^&*()_+-=[]{}|;:,.<>?");
    }
    
    // Remove ambiguous characters if requested
    if config.exclude_ambiguous {
        charset.retain(|&c| !b"0O1lI".contains(&c));
    }
    
    if charset.is_empty() {
        charset.extend_from_slice(b"abcdefghijklmnopqrstuvwxyz"); // fallback
    }
    
    let mut rng = rand::thread_rng();
    let mut password = Vec::new();
    
    // Ensure at least one character from each enabled set
    if config.include_lowercase && length > 0 {
        let lowercase: Vec<u8> = b"abcdefghijklmnopqrstuvwxyz".iter()
            .filter(|&&c| !config.exclude_ambiguous || !b"l".contains(&c))
            .copied().collect();
        if !lowercase.is_empty() {
            password.push(lowercase[rng.gen_range(0..lowercase.len())]);
        }
    }
    
    if config.include_uppercase && length > 1 {
        let uppercase: Vec<u8> = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ".iter()
            .filter(|&&c| !config.exclude_ambiguous || !b"OI".contains(&c))
            .copied().collect();
        if !uppercase.is_empty() {
            password.push(uppercase[rng.gen_range(0..uppercase.len())]);
        }
    }
    
    if config.include_numbers && length > 2 {
        let numbers: Vec<u8> = b"0123456789".iter()
            .filter(|&&c| !config.exclude_ambiguous || !b"01".contains(&c))
            .copied().collect();
        if !numbers.is_empty() {
            password.push(numbers[rng.gen_range(0..numbers.len())]);
        }
    }
    
    if config.include_symbols && length > 3 {
        password.push(b"!@#$%^&*"[rng.gen_range(0..8)]);
    }
    
    // Fill remaining length
    while password.len() < length {
        password.push(charset[rng.gen_range(0..charset.len())]);
    }
    
    // Shuffle the password to avoid predictable patterns
    use rand::seq::SliceRandom;
    password.shuffle(&mut rng);
    
    String::from_utf8(password).unwrap_or_else(|_| "password123".to_string())
}

// Generate memorable password (diceware-style)
pub fn generate_memorable_password(word_count: usize) -> String {
    const WORDS: &[&str] = &[
        "apple", "brave", "cloud", "dream", "eagle", "flame", "grace", "heart",
        "ivory", "jewel", "knight", "lemon", "magic", "noble", "ocean", "peace",
        "quiet", "river", "stone", "tiger", "unity", "voice", "water", "xenon",
        "youth", "zebra", "anchor", "bridge", "castle", "dragon", "empire", "forest"
    ];
    
    use rand::seq::SliceRandom;
    let mut rng = rand::thread_rng();
    
    (0..word_count)
        .map(|_| WORDS.choose(&mut rng).unwrap_or(&"word"))
        .map(|word| {
            let mut word = word.to_string();
            // Capitalize first letter
            if let Some(first_char) = word.chars().next() {
                word.replace_range(0..first_char.len_utf8(), &first_char.to_uppercase().to_string());
            }
            word
        })
        .collect::<Vec<_>>()
        .join("")
}

//! Utility Functions
//!
//! Provides password generation (random, configurable, memorable), password
//! strength analysis, clipboard helpers, and basic I/O helpers.

pub mod health;
pub mod import_export;
pub mod logging;

use std::io::{self, IsTerminal, Write};
use std::sync::LazyLock;

use clipboard::{ClipboardContext, ClipboardProvider};
use regex::Regex;
use zeroize::Zeroizing;

use crate::core::error::PassmanResult;

// ---------------------------------------------------------------------------
// Password generation configuration
// ---------------------------------------------------------------------------

/// Configuration for random password generation.
///
/// Named `PasswordGenConfig` to avoid collision with
/// [`crate::core::config::PasswordConfig`] (application-level password settings).
#[derive(Debug, Clone)]
pub struct PasswordGenConfig {
    pub include_uppercase: bool,
    pub include_lowercase: bool,
    pub include_numbers: bool,
    pub include_symbols: bool,
    /// Exclude visually ambiguous characters: `0 O l I 1`.
    pub exclude_ambiguous: bool,
}

impl Default for PasswordGenConfig {
    fn default() -> Self {
        Self {
            include_uppercase: true,
            include_lowercase: true,
            include_numbers: true,
            include_symbols: true,
            exclude_ambiguous: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Clipboard helpers
// ---------------------------------------------------------------------------

/// Copy text to the system clipboard.
pub fn copy_to_clipboard(text: &str) -> PassmanResult<()> {
    let mut ctx: ClipboardContext = ClipboardProvider::new()
        .map_err(|e| format!("clipboard init failed: {}", e))?;
    ctx.set_contents(text.to_owned())
        .map_err(|e| format!("clipboard set failed: {}", e))?;
    println!("✓ Copied to clipboard");
    Ok(())
}

// ---------------------------------------------------------------------------
// I/O helpers
// ---------------------------------------------------------------------------

/// Read a password securely from stdin (hides input in interactive mode).
pub fn read_password_secure(prompt: &str) -> PassmanResult<Zeroizing<String>> {
    print!("{}", prompt);
    io::stdout().flush()?;

    let password = if std::io::stdin().is_terminal() {
        Zeroizing::new(
            rpassword::read_password()
                .map_err(|e| format!("failed to read password: {}", e))?,
        )
    } else {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Zeroizing::new(input.trim().to_string())
    };

    if password.trim().is_empty() {
        return Err("Password cannot be empty".into());
    }
    Ok(password)
}

/// Read a non-empty line from stdin, re-prompting on empty input.
pub fn read_line(prompt: &str) -> PassmanResult<String> {
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

/// Read a line from stdin (empty input is allowed).
pub fn read_line_optional(prompt: &str) -> PassmanResult<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

// ---------------------------------------------------------------------------
// Password generation
// ---------------------------------------------------------------------------

/// Generate a random password of the given length using the full charset
/// (upper + lower + digits + symbols).
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

/// Generate a random password respecting the given [`PasswordGenConfig`].
///
/// Guarantees at least one character of each enabled category (when the
/// requested length permits) and shuffles the result.
pub fn generate_password_with_config(length: usize, config: &PasswordGenConfig) -> String {
    use rand::seq::SliceRandom;
    use rand::Rng;

    let mut charset = Vec::with_capacity(94); // ASCII printable max

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

    if config.exclude_ambiguous {
        charset.retain(|&c| !b"0O1lI".contains(&c));
    }

    if charset.is_empty() {
        charset.extend_from_slice(b"abcdefghijklmnopqrstuvwxyz"); // fallback
    }

    let mut rng = rand::thread_rng();
    let mut password = Vec::with_capacity(length);

    // Seed one character from each enabled category
    if config.include_lowercase && length > 0 {
        let set: Vec<u8> = b"abcdefghijklmnopqrstuvwxyz"
            .iter()
            .filter(|&&c| !config.exclude_ambiguous || !b"l".contains(&c))
            .copied()
            .collect();
        if !set.is_empty() {
            password.push(set[rng.gen_range(0..set.len())]);
        }
    }
    if config.include_uppercase && password.len() < length {
        let set: Vec<u8> = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"
            .iter()
            .filter(|&&c| !config.exclude_ambiguous || !b"OI".contains(&c))
            .copied()
            .collect();
        if !set.is_empty() {
            password.push(set[rng.gen_range(0..set.len())]);
        }
    }
    if config.include_numbers && password.len() < length {
        let set: Vec<u8> = b"0123456789"
            .iter()
            .filter(|&&c| !config.exclude_ambiguous || !b"01".contains(&c))
            .copied()
            .collect();
        if !set.is_empty() {
            password.push(set[rng.gen_range(0..set.len())]);
        }
    }
    if config.include_symbols && password.len() < length {
        password.push(b"!@#$%^&*"[rng.gen_range(0..8)]);
    }

    // Fill remaining length from the combined charset
    while password.len() < length {
        password.push(charset[rng.gen_range(0..charset.len())]);
    }

    password.shuffle(&mut rng);

    String::from_utf8(password).expect("BUG: password generator produced invalid UTF-8 from ASCII charset")
}

/// Word list for memorable (diceware-style) password generation.
const MEMORABLE_WORDS: &[&str] = &[
    "apple", "brave", "cloud", "dream", "eagle", "flame", "grace", "heart",
    "ivory", "jewel", "knight", "lemon", "magic", "noble", "ocean", "peace",
    "quiet", "river", "stone", "tiger", "unity", "voice", "water", "xenon",
    "youth", "zebra", "anchor", "bridge", "castle", "dragon", "empire", "forest",
    "galaxy", "harbor", "island", "jungle", "lantern", "marble", "nebula", "orchid",
    "palace", "quartz", "rainbow", "silver", "tornado", "umbrella", "valley", "winter",
];

/// Generate a memorable password by concatenating `word_count` random
/// capitalized words from a curated word list.
pub fn generate_memorable_password(word_count: usize) -> String {
    use rand::seq::SliceRandom;

    let mut rng = rand::thread_rng();

    (0..word_count)
        .map(|_| MEMORABLE_WORDS.choose(&mut rng).unwrap_or(&"word"))
        .map(|word| {
            let mut w = word.to_string();
            if let Some(first) = w.chars().next() {
                w.replace_range(0..first.len_utf8(), &first.to_uppercase().to_string());
            }
            w
        })
        .collect::<Vec<_>>()
        .join("")
}

// ---------------------------------------------------------------------------
// Password strength analysis
// ---------------------------------------------------------------------------

/// Password strength levels.
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
            Self::VeryWeak => write!(f, "Very Weak"),
            Self::Weak => write!(f, "Weak"),
            Self::Fair => write!(f, "Fair"),
            Self::Good => write!(f, "Good"),
            Self::Strong => write!(f, "Strong"),
        }
    }
}

/// Top-100 common passwords (truncated — full list would use a crate like
/// `zxcvbn`). Passwords that appear in this list incur a heavy penalty.
static COMMON_PASSWORDS: &[&str] = &[
    "password", "123456", "12345678", "password1", "password123",
    "admin", "qwerty", "letmein", "welcome", "monkey",
    "master", "dragon", "login", "princess", "football",
    "shadow", "sunshine", "trustno1", "iloveyou", "batman",
    "access", "hello", "charlie", "123456789", "1234567",
    "abc123", "654321", "michael", "jordan", "superman",
    "harley", "robert", "daniel", "matthew", "summer",
    "george", "jennifer", "hunter", "amanda", "michelle",
    "thomas", "andrew", "andrea", "joshua", "james",
    "nicole", "jessica", "freedom", "liverpool", "soccer",
    "passw0rd", "starwars", "ginger", "killer", "pepper",
    "ranger", "buster", "hockey", "zaq1zaq1", "mustang",
    "bailey", "nothing", "flower", "12345", "111111",
    "tigger", "joshua", "cookie", "richard", "dallas",
    "test", "test123", "pa$$word", "qwerty123", "asdf",
    "zxcvbnm", "q1w2e3r4", "1q2w3e4r", "1qaz2wsx", "pass",
    "changeme", "default", "root", "toor", "guest",
    "administrator", "p@ssw0rd", "p@ssword", "baseball", "basketball",
    "solo", "ashley", "thunder", "maggie", "oliver",
    "silver", "corvette", "diamond", "secret", "computer",
];

/// Analyze the strength of a password and return actionable suggestions.
pub fn analyze_password_strength(password: &str) -> (PasswordStrength, Vec<String>) {
    let mut score: i32 = 0;
    let mut suggestions = Vec::new();

    // --- Length ---
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

    // --- Character variety ---
    let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
    let has_numbers = password.chars().any(|c| c.is_ascii_digit());
    let has_symbols = password.chars().any(|c| !c.is_alphanumeric() && c.is_ascii());

    if has_lowercase { score += 1; } else { suggestions.push("Add lowercase letters".into()); }
    if has_uppercase { score += 1; } else { suggestions.push("Add uppercase letters".into()); }
    if has_numbers { score += 1; } else { suggestions.push("Add numbers".into()); }
    if has_symbols { score += 1; } else { suggestions.push("Add special characters".into()); }

    // --- Repeated characters (e.g. "aaa") ---
    let chars: Vec<char> = password.chars().collect();
    let has_repeated = chars
        .windows(3)
        .any(|w| w[0] == w[1] && w[1] == w[2]);
    if has_repeated {
        score -= 1;
        suggestions.push("Avoid repeating characters".into());
    }

    // --- Sequential patterns ---
    static SEQ_RE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"(?i)(012|123|234|345|456|567|678|789|890|abc|bcd|cde|def|efg|fgh|ghi|hij|ijk|jkl|klm|lmn|mno|nop|opq|pqr|qrs|rst|stu|tuv|uvw|vwx|wxy|xyz)",
        )
        .unwrap()
    });
    if SEQ_RE.is_match(password) {
        score -= 1;
        suggestions.push("Avoid sequential characters".into());
    }

    // --- Common passwords ---
    let lower = password.to_lowercase();
    if COMMON_PASSWORDS.iter().any(|&p| lower.contains(p)) {
        score -= 2;
        suggestions.push("Avoid common passwords".into());
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ============ Password Generation Tests ============

    #[test]
    fn test_generate_password_length() {
        for length in [8, 12, 16, 20, 32, 64] {
            let password = generate_password(length);
            assert_eq!(password.len(), length, "Password should be {} chars", length);
        }
    }

    #[test]
    fn test_generate_password_uniqueness() {
        let passwords: Vec<String> = (0..100).map(|_| generate_password(16)).collect();
        let unique_count = passwords.iter().collect::<std::collections::HashSet<_>>().len();
        assert_eq!(unique_count, 100, "All generated passwords should be unique");
    }

    #[test]
    fn test_generate_password_with_config_lowercase_only() {
        let config = PasswordGenConfig {
            include_lowercase: true,
            include_uppercase: false,
            include_numbers: false,
            include_symbols: false,
            exclude_ambiguous: false,
        };
        let password = generate_password_with_config(20, &config);
        assert!(
            password.chars().all(|c| c.is_ascii_lowercase()),
            "Password should contain only lowercase: {}",
            password
        );
    }

    #[test]
    fn test_generate_password_with_config_uppercase_only() {
        let config = PasswordGenConfig {
            include_lowercase: false,
            include_uppercase: true,
            include_numbers: false,
            include_symbols: false,
            exclude_ambiguous: false,
        };
        let password = generate_password_with_config(20, &config);
        assert!(
            password.chars().all(|c| c.is_ascii_uppercase()),
            "Password should contain only uppercase: {}",
            password
        );
    }

    #[test]
    fn test_generate_password_with_config_numbers_only() {
        let config = PasswordGenConfig {
            include_lowercase: false,
            include_uppercase: false,
            include_numbers: true,
            include_symbols: false,
            exclude_ambiguous: false,
        };
        let password = generate_password_with_config(20, &config);
        assert!(
            password.chars().all(|c| c.is_ascii_digit()),
            "Password should contain only numbers: {}",
            password
        );
    }

    #[test]
    fn test_generate_password_with_config_exclude_ambiguous() {
        let config = PasswordGenConfig {
            include_lowercase: true,
            include_uppercase: true,
            include_numbers: true,
            include_symbols: false,
            exclude_ambiguous: true,
        };
        let ambiguous_chars = ['0', 'O', '1', 'l', 'I'];

        for _ in 0..50 {
            let password = generate_password_with_config(32, &config);
            assert!(
                !password.chars().any(|c| ambiguous_chars.contains(&c)),
                "Password should not contain ambiguous chars: {}",
                password
            );
        }
    }

    #[test]
    fn test_generate_password_with_config_all_types() {
        let config = PasswordGenConfig {
            include_lowercase: true,
            include_uppercase: true,
            include_numbers: true,
            include_symbols: true,
            exclude_ambiguous: false,
        };

        let password = generate_password_with_config(32, &config);
        assert!(password.chars().any(|c| c.is_ascii_lowercase()), "Should have lowercase");
        assert!(password.chars().any(|c| c.is_ascii_uppercase()), "Should have uppercase");
        assert!(password.chars().any(|c| c.is_ascii_digit()), "Should have numbers");
        assert!(password.chars().any(|c| !c.is_alphanumeric()), "Should have symbols");
    }

    #[test]
    fn test_generate_password_empty_config_fallback() {
        let config = PasswordGenConfig {
            include_lowercase: false,
            include_uppercase: false,
            include_numbers: false,
            include_symbols: false,
            exclude_ambiguous: false,
        };
        let password = generate_password_with_config(16, &config);
        assert!(!password.is_empty(), "Should fallback to generating something");
    }

    // ============ Memorable Password Tests ============

    #[test]
    fn test_generate_memorable_password_word_count() {
        let password = generate_memorable_password(4);
        let capital_count = password.chars().filter(|c| c.is_uppercase()).count();
        assert_eq!(capital_count, 4, "Should have 4 capitalized words");
    }

    #[test]
    fn test_generate_memorable_password_uniqueness() {
        let passwords: Vec<String> = (0..50).map(|_| generate_memorable_password(4)).collect();
        let unique_count = passwords.iter().collect::<std::collections::HashSet<_>>().len();
        assert!(unique_count > 40, "Most passwords should be unique: {}/50", unique_count);
    }

    #[test]
    fn test_generate_memorable_password_readable() {
        let password = generate_memorable_password(3);
        assert!(
            password.chars().all(|c| c.is_alphabetic()),
            "Memorable password should be alphabetic: {}",
            password
        );
    }

    // ============ Password Strength Tests ============

    #[test]
    fn test_password_strength_very_weak() {
        let (strength, _) = analyze_password_strength("abc");
        assert_eq!(strength, PasswordStrength::VeryWeak);
    }

    #[test]
    fn test_password_strength_weak() {
        let (strength, _) = analyze_password_strength("password");
        assert!(
            matches!(strength, PasswordStrength::VeryWeak | PasswordStrength::Weak),
            "Common password should be weak: {:?}",
            strength
        );
    }

    #[test]
    fn test_password_strength_common_password_penalty() {
        let (strength, suggestions) = analyze_password_strength("password123");
        assert!(matches!(strength, PasswordStrength::VeryWeak | PasswordStrength::Weak));
        assert!(suggestions.iter().any(|s| s.contains("common")));
    }

    #[test]
    fn test_password_strength_sequential_penalty() {
        let (_, suggestions) = analyze_password_strength("abc123XYZ!");
        assert!(suggestions.iter().any(|s| s.contains("sequential")));
    }

    #[test]
    fn test_password_strength_repeated_chars_penalty() {
        let (_, suggestions) = analyze_password_strength("Hellooo123!");
        assert!(suggestions.iter().any(|s| s.contains("repeating")));
    }

    #[test]
    fn test_password_strength_strong() {
        let (strength, suggestions) = analyze_password_strength("Kj9$mP2!qR5@nL8*");
        assert_eq!(strength, PasswordStrength::Strong);
        assert!(
            suggestions.is_empty() || suggestions.len() <= 1,
            "Strong password should have few suggestions: {:?}",
            suggestions
        );
    }

    #[test]
    fn test_password_strength_suggestions_for_missing_types() {
        let (_, suggestions) = analyze_password_strength("onlylowercase");
        assert!(suggestions.iter().any(|s| s.contains("uppercase")));
        assert!(suggestions.iter().any(|s| s.contains("numbers")));
        assert!(suggestions.iter().any(|s| s.contains("special")));
    }

    #[test]
    fn test_password_strength_length_suggestions() {
        let (_, suggestions) = analyze_password_strength("Ab1!");
        assert!(suggestions.iter().any(|s| s.contains("8 characters")));
    }

    // ============ Config Tests ============

    #[test]
    fn test_password_gen_config_default() {
        let config = PasswordGenConfig::default();
        assert!(config.include_uppercase);
        assert!(config.include_lowercase);
        assert!(config.include_numbers);
        assert!(config.include_symbols);
        assert!(!config.exclude_ambiguous);
    }

    // ============ Password Strength Display Tests ============

    #[test]
    fn test_password_strength_display() {
        assert_eq!(format!("{}", PasswordStrength::VeryWeak), "Very Weak");
        assert_eq!(format!("{}", PasswordStrength::Weak), "Weak");
        assert_eq!(format!("{}", PasswordStrength::Fair), "Fair");
        assert_eq!(format!("{}", PasswordStrength::Good), "Good");
        assert_eq!(format!("{}", PasswordStrength::Strong), "Strong");
    }
}

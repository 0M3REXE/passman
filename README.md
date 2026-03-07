<div align="center">

**A secure password manager for Windows and Linux, built in Rust.**

[![CI](https://github.com/0M3REXE/passman/actions/workflows/ci.yml/badge.svg)](https://github.com/0M3REXE/passman/actions/workflows/ci.yml)
[![Release](https://github.com/0M3REXE/passman/actions/workflows/release.yml/badge.svg)](https://github.com/0M3REXE/passman/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-1.5.0-brightgreen.svg)](https://github.com/0M3REXE/passman/releases/tag/v1.5.0)
[![Rust](https://img.shields.io/badge/rust-2021-orange.svg)](https://www.rust-lang.org/)

[Installation](#installation) | [Usage](#usage) | [Security](#security-architecture) | [Screenshots](#screenshots)

</div>

---

## Overview

Passman is a self-contained password manager with dual GUI and CLI interfaces, featuring AES-256-GCM authenticated encryption, Argon2id key derivation, HMAC-SHA256 integrity verification, and automatic memory zeroization. All credentials are stored in a single encrypted vault file portable across Windows and Linux.

---

## Features

### Core
- Dual interface: modern immediate-mode GUI (egui) and fully-featured CLI (clap)
- AES-256-GCM authenticated encryption with per-entry random nonces
- Argon2id key derivation with configurable and floor-validated parameters
- HMAC-SHA256 integrity verification with constant-time comparison
- Domain-separated key hierarchy (encryption key + HMAC key via HKDF)
- Atomic file writes with automatic backup rotation

### Password Management
- Add, edit, view, search, and delete entries
- Entry fields: username, password, URL, tags, notes, TOTP secret, timestamps
- Tag-based filtering and full-text search
- Real-time password strength analysis with actionable suggestions
- Password health dashboard with breach detection and age tracking
- Random password generation with configurable charset and length
- Memorable passphrase generation (Diceware-style word lists)

### Clipboard Security
- Auto-clear after configurable timeout (default 30 seconds)
- UUID-based content tracking to avoid clearing unrelated content
- Windows: excludes passwords from clipboard history and cloud sync via native Win32 API
- Cross-platform support via arboard (X11, Wayland, Windows)

### Session and Access Control
- Brute-force protection with exponential backoff lockout
- Configurable max failed attempts (default 5)
- Auto-lock on inactivity with configurable timeout
- Master password policy enforcement (length, character requirements)

### Import and Export
- Export to JSON or CSV
- Import from JSON, CSV, Chrome, and Firefox password exports
- Merge mode for importing into existing vaults

### Vault Format (v4)
- Self-describing binary format with embedded Argon2 parameters
- Backward-compatible loading of v1, v2, and v3 vaults
- Automatic migration on save

### Configuration
- TOML-based configuration file with per-section settings
- Configurable Argon2 parameters, UI preferences, backup policy, and password generation defaults
- CLI commands to show, set, and reset configuration

---

## Security Architecture

### Encryption Pipeline

```
Master Password
       |
       v
  [ Argon2id ]  <-- random salt (stored in vault header)
       |            memory >= 32 MB, time >= 2, parallelism >= 1
       v
  Master Key (256-bit)
       |
       +--[ HKDF-Expand (SHA-256) ]--> Encryption Key  --> AES-256-GCM
       |
       +--[ HKDF-Expand (SHA-256) ]--> HMAC Key        --> HMAC-SHA256
```

### Vault File Layout

```
Offset  Field                   Size
------  ----------------------  --------
0       Magic ("PMAN")          4 bytes
4       Format version (4)      1 byte
5       Salt length             4 bytes (LE)
9       Argon2 memory (KB)      4 bytes (LE)
13      Argon2 time cost        4 bytes (LE)
17      Argon2 parallelism      4 bytes (LE)
21      Salt                    variable
        Nonce                   12 bytes
        HMAC-SHA256             32 bytes
        Ciphertext (AES-GCM)   variable
```

### Memory Protection

| Mechanism | Scope |
|-----------|-------|
| `ZeroizeOnDrop` | AES key material -- wiped when `Key` is dropped |
| `Zeroizing<String>` | Master password at all call sites |
| `SerializableSecret` | Entry passwords -- debug prints `[REDACTED]`, zeroized on drop |
| `OptionalSecret` | TOTP secrets -- same protections as above |
| Form field zeroization | GUI input fields cleared on vault lock |
| Serialized plaintext | Wrapped in `Zeroizing<Vec<u8>>`, wiped after encryption |

### Integrity Verification

- HMAC-SHA256 computed over `nonce || ciphertext` using a dedicated HMAC key
- Verified with constant-time comparison before decryption
- Prevents tampering and truncation attacks

---

## Installation

### Pre-built Binaries

Download from the [latest release](https://github.com/0M3REXE/passman/releases/latest) page, or use a script installer:

**Linux:**
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/0M3REXE/passman/releases/download/v1.5.0/passman-installer.sh | sh
```

**Windows (PowerShell):**
```powershell
irm https://github.com/0M3REXE/passman/releases/download/v1.5.0/passman-installer.ps1 | iex
```

A Windows MSI installer is also available on the release page.

### Build from Source

```bash
git clone https://github.com/0M3REXE/passman
cd passman
cargo build --release
```

The binary will be at `target/release/passman` (or `passman.exe` on Windows).

**Linux build dependencies:**
```bash
sudo apt-get install -y libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libx11-dev libssl-dev
```

### Supported Platforms

| Target | Installer |
|--------|-----------|
| x86_64 Windows | PowerShell, MSI |
| x86_64 Linux | Shell script |

---

## Usage

### GUI Mode

Run the executable without arguments to launch the graphical interface:

```bash
passman
```

The GUI provides a welcome screen for vault creation or login, a main screen with searchable entry list and one-click copy, entry add/edit forms with inline strength analysis, a password health dashboard, import/export dialogs with native file pickers, and a settings panel.

### CLI Mode

Pass a subcommand to operate in terminal mode. A `--vault <path>` flag is available on all commands to specify the vault file (default: `vault.dat`).

#### Vault Operations

```bash
passman init                           # Create a new vault
passman change-password                # Change master password
passman backup -o ./backups            # Create a backup
passman vaults                         # List vault files in current directory
```

#### Entry Management

```bash
passman add github                     # Add entry (interactive prompts)
passman add github -u user -g -l 24   # Add with username, generate 24-char password
passman get github                     # Display credentials
passman get github -c                  # Copy password to clipboard
passman edit github                    # Edit interactively
passman rm github                      # Remove (with confirmation)
passman rm github -f                   # Remove without confirmation
```

#### Search and Analysis

```bash
passman list                           # List all entries
passman list -s "git" -t work          # Search by text, filter by tag
passman check                          # Check strength of a typed password
passman check -a                       # Analyze all vault passwords
passman health                         # Password health dashboard
passman health --issues-only           # Show only entries with issues
```

#### Password Generation

```bash
passman generate                       # Random 16-char password
passman generate -l 32 --symbols      # 32-char with symbols
passman generate --no-ambiguous        # Exclude 0, O, l, I, 1
passman generate -m                    # Memorable passphrase (4 words)
```

#### Import and Export

```bash
passman transfer export -o backup.json -f json
passman transfer export -o backup.csv -f csv
passman transfer import -i chrome.json -f chrome --merge
passman transfer import -i passwords.csv -f csv
```

#### Configuration

```bash
passman config show                           # Display current config
passman config set security.lock_timeout 600  # Set auto-lock to 10 min
passman config reset                          # Reset to defaults
```

---

## Configuration

Passman uses a TOML configuration file located at:
- **Windows:** `%APPDATA%\passman\passman.toml`
- **Linux:** `~/.config/passman/passman.toml`

Falls back to `passman.toml` in the working directory if the platform path is unavailable.

| Section | Key | Default | Description |
|---------|-----|---------|-------------|
| `general` | `default_vault` | `vault.dat` | Default vault file path |
| | `enable_logging` | `true` | Enable file logging |
| | `log_level` | `info` | Log verbosity (error, warn, info, debug, trace) |
| `security` | `lock_timeout_secs` | `300` | Auto-lock timeout in seconds (0 to disable) |
| | `clipboard_timeout_secs` | `30` | Clipboard auto-clear timeout |
| | `max_failed_attempts` | `5` | Failed attempts before lockout |
| | `min_password_length` | `12` | Minimum master password length |
| | `argon2_memory_kb` | `65536` | Argon2 memory parameter (KB) |
| | `argon2_time_cost` | `3` | Argon2 iteration count |
| | `argon2_parallelism` | `4` | Argon2 thread count |
| `password` | `default_length` | `20` | Default generated password length |
| | `include_symbols` | `true` | Include symbols in generation |
| | `exclude_ambiguous` | `false` | Exclude ambiguous characters |
| | `memorable_word_count` | `4` | Word count for passphrases |
| `backup` | `auto_backup` | `true` | Backup before each save |
| | `max_backups` | `10` | Maximum backup files retained |

---

## Screenshots

| | |
|---|---|
| ![Welcome Page](pictures/Screenshot%202025-12-11%20191320.png) | ![Welcome page 2](pictures/Screenshot%202025-12-11%20191338.png) |
| ![Create New Vault](pictures/Screenshot%202025-12-11%20191649.png) | ![Password Vault](pictures/Screenshot%202025-12-11%20191452.png) |
| ![Adding New Entry](pictures/Screenshot%202025-12-11%20191503.png) | ![Adding New Entry 1](pictures/Screenshot%202025-12-11%20191534.png) |
| ![Password Vault with Entry](pictures/Screenshot%202025-12-11%20191544.png) | |

---

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

---

<div align="center">

![Visitor Count](https://komarev.com/ghpvc/?username=0M3REXE&repo=passman&style=for-the-badge&color=brightgreen)

Created by [0M3REXE](https://github.com/0M3REXE)

</div>

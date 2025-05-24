# Passman - A Secure Password Manager

A command-line password manager written in Rust with AES-256-GCM encryption and Argon2 key derivation.

## Features

- **Secure Encryption**: Uses AES-256-GCM for encryption with Argon2 for key derivation
- **CLI Interface**: Simple command-line interface for managing passwords
- **Password Generation**: Built-in secure password generator
- **Cross-Platform**: Works on Windows, macOS, and Linux

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd passman
```

2. Build the project:
```bash
cargo build --release
```

3. The executable will be available at `target/release/passman.exe` (Windows) or `target/release/passman` (Unix)

## Usage

### Initialize Vault
Set up a master password and create an encrypted vault:
```bash
passman init
```

### Add Entry
Add a new password entry (interactive):
```bash
passman add github
```

### List Entries
List all saved entries:
```bash
passman list
```

### Get Entry
Print or copy credentials for an entry:
```bash
passman get github
```

### Remove Entry
Remove an entry from the vault:
```bash
passman rm github
```

## Security

- **Master Password**: Your vault is protected by a master password
- **Encryption**: All data is encrypted using AES-256-GCM
- **Key Derivation**: Uses Argon2 for secure key derivation from passwords
- **No Plaintext Storage**: Passwords are never stored in plaintext

## License

This project is licensed under the MIT License.

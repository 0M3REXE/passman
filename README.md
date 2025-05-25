
  _____                                    
 |  __ \                                   
 | |__) |_ _ ___ ___ _ __ ___   __ _ _ __  
 |  ___/ _` / __/ __| '_ ` _ \ / _` | '_ \ 
 | |  | (_| \__ \__ \ | | | | | (_| | | | |
 |_|   \__,_|___/___/_| |_| |_|\__,_|_| |_|
                                           
                                           
# Passman - A Secure Password Manager

A modern password manager written in Rust with both GUI and CLI interfaces, featuring AES-256-GCM encryption and Argon2 key derivation.

## Features

- **Dual Interface**: Modern GUI and powerful CLI interface
- **Secure Encryption**: Uses AES-256-GCM for encryption with Argon2 key derivation
- **Password Generation**: Built-in secure password generator with customizable length
- **Password Strength Analysis**: Real-time password strength analysis with suggestions
- **Multiple Vaults**: Support for multiple vault files
- **Clipboard Integration**: Secure clipboard operations for passwords
- **Cross-Platform**: Works on Windows, macOS, and Linux
- **Zero Dependencies**: Self-contained executable with no external dependencies

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

### GUI Mode (Default)
Simply run the executable without arguments to launch the GUI:
```bash
./passman
```

The GUI provides:
- **Welcome Screen**: Choose to create new vault or login to existing
- **Vault Management**: Create and manage multiple vault files
- **Password Management**: Add, edit, view, and delete password entries
- **Password Generator**: Generate secure passwords with customizable settings
- **Search & Filter**: Quickly find entries with real-time search
- **Clipboard Integration**: One-click copying of passwords
- **Password Visibility**: Toggle password visibility with eye icons

### CLI Mode
Pass any command-line argument to use CLI mode:

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
![Visitor Count](https://komarev.com/ghpvc/?username=0M3REXE&repo=your-repo&style=for-the-badge&color=brightgreen)


This project is licensed under the MIT License.

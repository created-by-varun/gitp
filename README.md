# gitp - Git Profile Switcher

A fast and intuitive git profile switcher written in Rust, with support for SSH keys, HTTPS credentials, and both global and local configurations.

## Features

- 🚀 **Fast** - Written in Rust for blazing-fast profile switching
- 🔑 **SSH Support** - Manage different SSH keys for different profiles
- 🌐 **Multi-scope** - Switch profiles globally or per-repository
- 🎨 **User-friendly** - Intuitive CLI with colored output
- 🔒 **Secure** - HTTPS credentials can be stored in the system keychain
- 🤖 **Auto-switching** - Automatically switch profiles based on repo URL (coming soon)

## Installation

### From source

```bash
# Make sure you have Rust installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and install
git clone https://github.com/created-by-varun/gitp.git
cd gitp
cargo install --path .
```

### From crates.io (coming soon)

```bash
cargo install gitp
```

## Quick Start

1. **Create a profile**:

   ```bash
   gitp new work --interactive
   # Or with flags
   gitp new personal --email "me@example.com" --name "John Doe"
   ```

2. **List profiles**:

   ```bash
   gitp list
   ```

3. **Switch profiles**:

   ```bash
   # Switch globally
   gitp use work

   # Switch for current repo only
   gitp use personal --local
   ```

4. **Check current profile**:
   ```bash
   gitp current
   ```

## Usage

### Profile Management

```bash
# Create a new profile interactively
gitp new <profile-name> --interactive

# Create with specific settings
gitp new work \
  --name "John Doe" \
  --email "john@company.com" \
  --ssh-key ~/.ssh/id_rsa_work

# Show profile details
gitp show work

# Edit existing profile
gitp edit work # Opens interactive mode

# Edit specific fields, including HTTPS credentials:
gitp edit work \
  --https-host github.com \
  --https-username myuser \
  --https-token "ghp_xxxxxxxxxxxxxxxxxxxx" \
  --https-store-in-keychain # Store the token in system keychain

# Remove HTTPS credentials from a profile (and keychain if stored there):
gitp edit work --https-remove-credentials

# Remove a profile
gitp remove work

# Rename a profile
gitp rename work work-backup
```

### Profile Switching

```bash
# Switch globally (default)
gitp use personal

# Switch for current repository only
gitp use work --local

# Show current profile
gitp current
gitp current --show-config  # With full configuration
```

## Configuration

Profiles are stored in `~/.config/gitp/config.toml`:

```toml
current_profile = "work"

[profiles.work]
name = "John Doe"
email = "john@company.com"
ssh_key = "~/.ssh/id_rsa_work"
gpg_key = "ABCD1234"

[profiles.personal]
name = "John Doe"
email = "john@personal.com"
ssh_key = "~/.ssh/id_rsa_personal"
```

## Roadmap

- [x] Basic profile management (create, list, switch, remove)
- [x] Global git config switching
- [x] SSH key management and SSH config generation
- [x] Local (per-repository) profile switching
- [x] HTTPS credential management via system keychain
- [ ] Auto-switching based on repository URLs
- [ ] Profile templates and inheritance
- [x] Import/export profiles
- [ ] Shell prompt integration
- [ ] Homebrew formula and AUR package

## Development

### Project Structure

```
gitp/
├── src/
│   ├── main.rs          # Entry point
│   ├── cli.rs           # CLI definitions
│   ├── commands/        # Command implementations
│   ├── config/          # Configuration management
│   ├── credentials/     # Credential management (keychain, tokens)
│   ├── git/             # Git operations
│   ├── ssh/             # SSH configuration management
│   └── utils/           # Utilities
├── tests/               # Integration tests
└── Cargo.toml
```

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with verbose output
cargo run -- --verbose list
```

### Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT).

## Acknowledgments

- Inspired by various git profile switchers in the ecosystem
- Built with excellent Rust crates: clap, serde, git2, and more

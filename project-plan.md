# gitp - Git Profile Switcher Project Plan

## Project Overview

A Rust CLI tool for managing and switching between multiple git profiles with support for SSH keys, HTTPS credentials, and both global and local configurations.

## Core Features

### Phase 1: MVP (Basic Profile Management)

- Create, list, edit, and delete profiles
- Switch between profiles (global config only)
- Store basic git config (user.name, user.email)
- Simple TOML/JSON configuration file

### Phase 2: SSH Support

- Link SSH keys to profiles
- SSH config generation/updating
- Support for multiple git hosts
- SSH agent integration (optional)

### Phase 3: Enhanced Features

- HTTPS credential management (using system keychain)
- GPG signing key support
- Local (repository-specific) profile application
- Auto-switching based on repo URL patterns

### Phase 4: Advanced Features

- Profile templates and inheritance
- Team profile sharing (import/export)
- Integration with git hooks
- Shell prompt integration

## Project Structure

```
gitp/
├── Cargo.toml
├── README.md
├── LICENSE
├── .github/
│   └── workflows/
│       ├── ci.yml
│       └── release.yml
├── src/
│   ├── main.rs           # CLI entry point
│   ├── cli.rs            # CLI argument parsing (clap)
│   ├── config/
│   │   ├── mod.rs        # Configuration management
│   │   ├── profile.rs    # Profile data structure
│   │   └── storage.rs    # Config file I/O
│   ├── commands/
│   │   ├── mod.rs        # Command trait and common logic
│   │   ├── new.rs        # Create new profile
│   │   ├── list.rs       # List profiles
│   │   ├── use.rs        # Switch profiles
│   │   ├── current.rs    # Show current profile
│   │   ├── edit.rs       # Edit existing profile
│   │   ├── remove.rs     # Delete profile
│   │   └── show.rs       # Show profile details
│   ├── git/
│   │   ├── mod.rs        # Git operations
│   │   ├── config.rs     # Git config manipulation
│   │   └── repo.rs       # Repository detection
│   ├── ssh/
│   │   ├── mod.rs        # SSH key management
│   │   ├── agent.rs      # SSH agent interaction
│   │   └── config.rs     # SSH config file management
│   ├── credentials/
│   │   ├── mod.rs        # Credential management
│   │   └── keyring.rs    # System keychain integration
│   └── utils/
│       ├── mod.rs        # Utility functions
│       └── error.rs      # Error types
├── tests/
│   ├── integration/      # Integration tests
│   └── fixtures/         # Test data
└── examples/             # Example usage

```

## Data Models

```rust
// Profile structure
pub struct Profile {
    pub name: String,
    pub git_config: GitConfig,
    pub ssh_key: Option<PathBuf>,
    pub gpg_key: Option<String>,
    pub https_credentials: Option<HttpsCredentials>,
    pub custom_config: HashMap<String, String>,
}

pub struct GitConfig {
    pub user_name: String,
    pub user_email: String,
    pub user_signingkey: Option<String>,
    // Additional git config options
}

pub struct HttpsCredentials {
    pub host: String,
    pub username: String,
    pub credential_type: CredentialType,
}

pub enum CredentialType {
    Token(String),        // Personal access token
    KeychainRef(String),  // Reference to system keychain
}

// Configuration file structure
pub struct Config {
    pub profiles: HashMap<String, Profile>,
    pub current_profile: Option<String>,
    pub auto_switch_rules: Vec<AutoSwitchRule>,
}

pub struct AutoSwitchRule {
    pub pattern: String,  // e.g., "github.com/company/*"
    pub profile_name: String,
}
```

## CLI Interface Design

```bash
# Profile management
gitp new <name>                    # Interactive profile creation
gitp new work --email work@company.com --name "John Doe" --ssh-key ~/.ssh/id_rsa_work
gitp list                          # List all profiles
gitp show <name>                   # Show profile details
gitp edit <name>                   # Edit profile interactively
gitp remove <name>                 # Delete profile
gitp rename <old-name> <new-name>  # Rename profile

# Profile switching
gitp use <name>                    # Switch globally
gitp use <name> --local            # Switch for current repo only
gitp use <name> --global           # Explicit global switch
gitp current                       # Show current profile
gitp current --show-config         # Show current profile with all settings

# SSH management
gitp ssh-add <profile> <key-path>  # Add SSH key to profile
gitp ssh-config generate           # Generate SSH config entries

# Auto-switching
gitp auto add <pattern> <profile>  # Add auto-switch rule
gitp auto list                     # List auto-switch rules
gitp auto remove <pattern>         # Remove auto-switch rule

# Import/Export
gitp export <profile> > profile.toml
gitp import < profile.toml
```

## Configuration Storage

Default location: `~/.config/gitp/config.toml`

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

[[auto_switch]]
pattern = "github.com/company/*"
profile = "work"

[[auto_switch]]
pattern = "github.com/johndoe/*"
profile = "personal"
```

## Dependencies

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
git2 = "0.18"
dirs = "5"           # For config directory
colored = "2"        # For colored output
dialoguer = "0.11"   # For interactive prompts
thiserror = "1"      # For error handling
anyhow = "1"         # For error handling
keyring = "2"        # For credential storage

[dev-dependencies]
tempfile = "3"       # For testing
assert_cmd = "2"     # For CLI testing
predicates = "3"     # For test assertions
```

## Implementation Phases

### Phase 1: MVP (Week 1-2)

1. Set up project structure and CI
2. Implement basic CLI with clap
3. Create profile data structures
4. Implement config file storage
5. Basic commands: new, list, use, current
6. Global git config switching

### Phase 2: Enhanced Profile Management (Week 3)

1. Interactive profile creation/editing
2. Profile validation
3. Show and remove commands
4. Better error handling and user feedback
5. Colored output and formatting

### Phase 3: SSH Support (Week 4-5)

1. SSH key association with profiles
2. SSH config file generation
3. Multiple host support
4. Testing with different git providers

### Phase 4: Advanced Features (Week 6-7)

1. Local repository profile switching
2. Auto-switching based on remote URLs
3. HTTPS credential management
4. Import/export functionality

### Phase 5: Polish & Release (Week 8)

1. Comprehensive testing
2. Documentation
3. Installation scripts
4. Homebrew formula / AUR package
5. GitHub releases with binaries

## Testing Strategy

1. **Unit Tests**: For individual functions and modules
2. **Integration Tests**: For command execution and file operations
3. **E2E Tests**: Full workflow scenarios
4. **Manual Testing**: With real git repositories and SSH keys

## Success Metrics

- Fast profile switching (<100ms)
- Zero data loss during operations
- Cross-platform compatibility (Linux, macOS, Windows)
- Intuitive CLI interface
- Comprehensive error messages
- Minimal dependencies

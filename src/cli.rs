use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "gitp",
    about = "A fast git profile switcher with SSH and HTTPS support",
    version,
    author,
    long_about = None
)]
pub struct Cli {
    /// Turn on/off colored output
    #[arg(long, global = true, default_value = "true")]
    pub color: bool,

    /// Increase verbosity
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new profile
    New {
        /// Profile name
        name: String,

        /// Git user name (for non-interactive mode)
        #[arg(long)]
        user_name: Option<String>,

        /// Git user email (for non-interactive mode)
        #[arg(long)]
        user_email: Option<String>,

        /// Git signing key (for non-interactive mode)
        #[arg(long)]
        signing_key: Option<String>,

        /// Path to the SSH key (for non-interactive mode)
        #[arg(long)]
        ssh_key_path: Option<String>,

        /// GPG key ID for signing (for non-interactive mode)
        #[arg(long)]
        gpg_key_id: Option<String>,

        /// Hostname for the SSH key (e.g., github.com, requires --ssh-key-path)
        #[arg(long, requires = "ssh_key_path")]
        ssh_key_host: Option<String>,

        // HTTPS Credentials (for non-interactive mode)
        /// Hostname for HTTPS (e.g., github.com).
        #[arg(long, group = "https_new_details")]
        https_host: Option<String>,
        /// Username for HTTPS (requires --https-host).
        #[arg(long, requires = "https_host")]
        https_username: Option<String>,
        /// Token for HTTPS (requires --https-host and --https-username; conflicts with --https-keychain-ref).
        #[arg(long, requires_all = ["https_host", "https_username"], conflicts_with = "https_keychain_ref")]
        https_token: Option<String>,
        /// Keychain reference for HTTPS (requires --https-host and --https-username; conflicts with --https-token).
        #[arg(long, requires_all = ["https_host", "https_username"], conflicts_with = "https_token")]
        https_keychain_ref: Option<String>,
    },

    /// List all profiles
    List {
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Switch to a profile
    #[command(name = "use")]
    Use {
        /// Profile name
        name: String,

        /// Apply profile to current repository only
        #[arg(short, long, conflicts_with = "global")]
        local: bool,

        /// Apply profile globally (default behavior)
        #[arg(short, long)]
        global: bool,
    },

    /// Show profile details
    Show {
        /// Profile name
        name: String,
    },

    /// Edit an existing profile
    Edit {
        /// Profile name
        name: String,

        /// New Git user name (for non-interactive mode)
        #[arg(long)]
        user_name: Option<String>,

        /// New Git user email (for non-interactive mode)
        #[arg(long)]
        user_email: Option<String>,

        /// New Git signing key (for non-interactive mode)
        #[arg(long)]
        signing_key: Option<String>,

        /// New path to the SSH key (for non-interactive mode)
        #[arg(long)]
        ssh_key_path: Option<String>,

        /// New GPG key ID for signing (for non-interactive mode)
        #[arg(long)]
        gpg_key_id: Option<String>,

        /// New hostname for the SSH key (e.g., github.com, requires --ssh-key-path)
        /// To remove, provide an empty string if --ssh-key-path is also specified.
        #[arg(long, requires = "ssh_key_path")]
        ssh_key_host: Option<String>,

        // HTTPS Credentials (for non-interactive mode)
        /// New hostname for HTTPS (e.g., github.com).
        #[arg(long, group = "https_edit_details")]
        https_host: Option<String>,
        /// New username for HTTPS (requires --https-host).
        #[arg(long, requires = "https_host")]
        https_username: Option<String>,
        /// New token for HTTPS (requires --https-host and --https-username; conflicts with --https-keychain-ref).
        /// To remove, provide an empty string with --https-token \"\" if host and username are specified.
        #[arg(long, requires_all = ["https_host", "https_username"], conflicts_with = "https_keychain_ref")]
        https_token: Option<String>,
        /// New keychain reference for HTTPS (requires --https-host and --https-username; conflicts with --https-token).
        /// To remove, provide an empty string with --https-keychain-ref \"\" if host and username are specified.
        #[arg(long, requires_all = ["https_host", "https_username"], conflicts_with = "https_token")]
        https_keychain_ref: Option<String>,
    },

    /// Remove a profile
    Remove {
        /// Profile name
        name: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// Rename a profile
    Rename {
        /// Current profile name
        old_name: String,

        /// New profile name
        new_name: String,
    },
    // Future commands to be added:
    /// Manage SSH keys associated with profiles
    SshKey {
        #[command(subcommand)]
        command: SshKeyCommands,
    },
    /// Display the current Git user name, email, and signing key
    Current,
    /// Export a profile to a TOML file or stdout
    Export {
        /// Name of the profile to export
        name: String,

        /// Optional path to save the exported profile (e.g., profile.toml).
        /// If not provided, the profile will be printed to stdout.
        #[arg(short, long)]
        output_path: Option<String>,
    },

    /// Import a profile from a TOML file or stdin
    Import {
        /// Path to the TOML file to import the profile from.
        /// Use "-" or omit to read from stdin.
        #[arg(default_value = "-")]
        input_path: String, // clap handles default_value, so String is fine

        /// Optional name to save the imported profile as.
        /// If not provided, uses the 'name' field from the imported file.
        #[arg(short, long)]
        profile_name: Option<String>,

        /// Overwrite existing profile if it has the same name
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum SshKeyCommands {
    /// Set or update the SSH key path for a profile
    Set {
        /// Name of the profile
        profile_name: String,
        /// Path to the SSH private key (e.g., ~/.ssh/id_rsa_work)
        key_path: String,
    },
    /// Remove the SSH key association from a profile
    Remove {
        /// Name of the profile
        profile_name: String,
    },
    /// Show the SSH key path associated with a profile
    Show {
        /// Name of the profile
        profile_name: String,
    },
}

// For future implementation
// #[derive(Subcommand)]
// pub enum SshConfigCommands { // Renamed from SshConfigAction for clarity
//     // Define actions like GenerateHostEntry, RemoveHostEntry etc.
// }

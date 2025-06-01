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
    // /// Manage SSH keys (TODO: further define subcommands like add, list, remove)
    // SshKey { #[command(subcommand)] command: SshKeyCommands },
    /// Display the current Git user name, email, and signing key
    Current,
    // /// Export a profile
    // Export {
    //     /// Profile name
    //     name: String,
    // },

    // /// Import a profile
    // Import {
    //     /// Path to profile file
    //     #[arg(default_value = "-")]
    //     path: String,
    // },
}

// For future implementation
// #[derive(Subcommand)]
// pub enum SshCommands {
//     /// Add SSH key to profile
//     Add {
//         /// Profile name
//         profile: String,
//         /// SSH key path
//         key_path: String,
//     },
//     /// Generate SSH config
//     Config {
//         /// Generate config entries
//         #[command(subcommand)]
//         action: SshConfigAction,
//     },
// }

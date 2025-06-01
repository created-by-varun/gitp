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

        /// Use interactive mode
        #[arg(short, long)]
        interactive: bool,
        // These will be added in Phase 1 implementation
        // /// Git user name
        // #[arg(long)]
        // git_name: Option<String>,

        // /// Git user email
        // #[arg(long)]
        // email: Option<String>,

        // /// SSH key path
        // #[arg(long)]
        // ssh_key: Option<String>,
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

    /// Show current active profile
    Current {
        /// Show full configuration
        #[arg(short = 'c', long)]
        show_config: bool,
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
    // /// Manage SSH configurations
    // Ssh {
    //     #[command(subcommand)]
    //     command: SshCommands,
    // },

    // /// Manage auto-switching rules
    // Auto {
    //     #[command(subcommand)]
    //     command: AutoCommands,
    // },

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

// src/commands/use_profile.rs
use anyhow::{bail, Context, Result};
use colored::Colorize;

use crate::config::Config;

// The `local` and `global` flags will be used when Git interaction is implemented.
#[allow(unused_variables)]
pub fn execute(name: String, local: bool, global: bool) -> Result<()> {
    let mut config = Config::load().context("Failed to load configuration.")?;

    if !config.profiles.contains_key(&name) {
        bail!(
            "Profile '{}' not found. Use '{}' to list available profiles or '{}' to create a new one.",
            name.yellow(),
            "gitp list".cyan(),
            format!("gitp new {}", name).cyan()
        );
    }

    config.current_profile = Some(name.clone());
    config.save().context("Failed to save configuration.")?;

    println!("Successfully switched to profile: {}", name.green());
    println!(
        "{}",
        "Note: Actual Git configuration changes (local/global) are not yet implemented.".dimmed()
    );

    // Future: Implement actual git config changes based on `local` and `global` flags.
    // if local {
    //     println!("Applying profile '{}' to the current repository (local).", name);
    //     // Call git interaction logic for local scope
    // } else {
    //     // Default to global if neither or only global is specified, or handle explicit global.
    //     println!("Applying profile '{}' globally.", name);
    //     // Call git interaction logic for global scope
    // }

    Ok(())
}

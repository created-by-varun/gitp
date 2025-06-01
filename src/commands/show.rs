use anyhow::{bail, Context, Result};
use colored::Colorize;

use crate::commands::list::print_profile_detailed;
use crate::config::Config; // Import the shared function

pub fn execute(name: String) -> Result<()> {
    let config = Config::load().context("Failed to load configuration.")?;

    if let Some(profile_details) = config.profiles.get(&name) {
        println!("Details for profile: {}", name.cyan().bold());
        // Pass config.current_profile.as_deref() to correctly show if it's the current one
        print_profile_detailed(&name, profile_details, config.current_profile.as_deref());
    } else {
        bail!(
            "Profile '{}' not found. Use '{}' to list available profiles.",
            name.yellow(),
            "gitp list".cyan()
        );
    }

    Ok(())
}

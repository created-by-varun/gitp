use anyhow::{Context, Result};
use colored::Colorize;

use crate::config::Config;
use crate::commands::list::print_profile_detailed; // Import the shared function

pub fn execute(show_config: bool) -> Result<()> {
    let config = Config::load().context("Failed to load configuration.")?;

    if let Some(profile_name) = &config.current_profile {
        if show_config {
            println!("Current active profile (full configuration):");
            if let Some(profile_details) = config.profiles.get(profile_name) {
                // Pass Some(profile_name) as current_profile to ensure it's highlighted correctly
                print_profile_detailed(profile_name, profile_details, Some(profile_name));
            } else {
                // This case should ideally not happen if current_profile is set and valid
                println!(
                    "{}",
                    format!(
                        "Warning: Current profile '{}' is set but its details were not found in the configuration.",
                        profile_name
                    ).yellow()
                );
            }
        } else {
            println!("Current active profile: {}", profile_name.green().bold());
        }
    } else {
        println!(
            "No profile is currently active. Use '{}' to activate one.",
            "gitp use <profile_name>".cyan()
        );
    }

    Ok(())
}

use anyhow::{bail, Context, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm};

use crate::config::Config;

pub fn execute(name: String, force: bool) -> Result<()> {
    let mut config = Config::load().context("Failed to load configuration.")?;

    if !config.profiles.contains_key(&name) {
        bail!("Profile '{}' not found. Cannot remove it.", name.yellow());
    }

    if !force {
        let confirmation = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Are you sure you want to remove profile '{}'?",
                name.yellow()
            ))
            .default(false) // Default to No
            .interact()
            .context("Failed to get confirmation for removal.")?;

        if !confirmation {
            println!("Removal of profile '{}' cancelled.", name.cyan());
            return Ok(());
        }
    }

    // Remove the profile from the HashMap
    config.profiles.remove(&name);

    // If the removed profile was the current one, unset it
    if config.current_profile.as_deref() == Some(name.as_str()) {
        config.current_profile = None;
        println!(
            "Profile '{}' was the current profile and has been unset.",
            name.yellow()
        );
    }

    config
        .save()
        .context("Failed to save configuration after removing profile.")?;

    println!("Profile '{}' removed successfully.", name.green());

    Ok(())
}

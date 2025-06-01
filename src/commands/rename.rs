use anyhow::{bail, Context, Result};
use colored::Colorize;

use crate::config::Config;

pub fn execute(old_name: String, new_name: String) -> Result<()> {
    let mut config = Config::load().context("Failed to load configuration.")?;

    if new_name.trim().is_empty() {
        bail!("New profile name cannot be empty.");
    }

    if !config.profiles.contains_key(&old_name) {
        bail!(
            "Profile '{}' not found. Cannot rename it.",
            old_name.yellow()
        );
    }

    if old_name == new_name {
        println!("The new name is the same as the old name. No changes made.");
        return Ok(());
    }

    if config.profiles.contains_key(&new_name) {
        bail!(
            "A profile named '{}' already exists. Please choose a different name.",
            new_name.yellow()
        );
    }

    // Remove the old profile, update its name, and insert it with the new name
    if let Some(mut profile_to_rename) = config.profiles.remove(&old_name) {
        profile_to_rename.name = new_name.clone();
        config.profiles.insert(new_name.clone(), profile_to_rename);

        // If the renamed profile was the current one, update current_profile
        if config.current_profile.as_deref() == Some(old_name.as_str()) {
            config.current_profile = Some(new_name.clone());
            println!(
                "Current profile '{}' has been updated to '{}'.",
                old_name.yellow(),
                new_name.green()
            );
        }

        config
            .save()
            .context("Failed to save configuration after renaming profile.")?;

        println!(
            "Profile '{}' successfully renamed to '{}'.",
            old_name.yellow(),
            new_name.green()
        );
    } else {
        // This case should ideally be caught by the contains_key check earlier,
        // but it's good practice for robustness if remove somehow fails after a successful check.
        bail!("Failed to retrieve profile '{}' for renaming, though it was initially found. This should not happen.", old_name.red());
    }

    Ok(())
}

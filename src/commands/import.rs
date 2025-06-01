use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::fs;
use std::io::{self, Read};

use crate::config::{Config, Profile};

pub fn execute(
    input_path: String,
    profile_name_override: Option<String>,
    force: bool,
) -> Result<()> {
    let mut input_content = String::new();

    if input_path == "-" {
        io::stdin()
            .read_to_string(&mut input_content)
            .context("Failed to read profile data from stdin.")?;
    } else {
        input_content = fs::read_to_string(&input_path)
            .with_context(|| format!("Failed to read profile data from file '{}'", input_path))?;
    }

    if input_content.trim().is_empty() {
        bail!("Import data is empty. Nothing to import.");
    }

    let mut imported_profile: Profile =
        toml::from_str(&input_content).context("Failed to deserialize profile from TOML data.")?;

    let final_profile_name = match profile_name_override {
        Some(name_override) => {
            if name_override.trim().is_empty() {
                bail!("Provided profile name override cannot be empty.");
            }
            // Update the name within the profile struct itself if a new name is given
            imported_profile.name = name_override.clone();
            name_override
        }
        None => {
            if imported_profile.name.trim().is_empty() {
                bail!(
                    "Profile name in the imported file cannot be empty if no override is provided."
                );
            }
            imported_profile.name.clone()
        }
    };

    // Validate the imported profile (after name is finalized)
    imported_profile
        .validate()
        .map_err(|e| anyhow::anyhow!(e)) // Convert ValidationError to anyhow::Error
        .context("Imported profile data is invalid.")?;

    let mut config = Config::load().context("Failed to load current configuration.")?;

    if !force && config.profiles.contains_key(&final_profile_name) {
        bail!(
            "A profile named '{}' already exists. Use --force to overwrite.",
            final_profile_name.yellow()
        );
    }

    config
        .profiles
        .insert(final_profile_name.clone(), imported_profile);
    config
        .save()
        .context("Failed to save configuration after importing profile.")?;

    println!(
        "Profile '{}' imported successfully.",
        final_profile_name.cyan()
    );

    Ok(())
}

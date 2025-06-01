use anyhow::{bail, Context, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Input};
use std::path::PathBuf;

use crate::config::Config;

pub fn execute(name: String) -> Result<()> {
    let mut config = Config::load().context("Failed to load configuration.")?;

    let profile_to_edit = config
        .profiles
        .get_mut(&name)
        .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found.", name.cyan()))?;

    println!("Editing profile: {}", name.cyan().bold());
    println!("{}", "(Press Enter to keep current value, if any)".dimmed());
    println!("{}", "Note: HTTPS credentials and custom config key-value pairs are not editable in this version.".dimmed());
    println!();

    // User Name
    let new_user_name = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("User name")
        .default(profile_to_edit.git_config.user_name.clone())
        .interact_text()
        .context("Failed to get user name input.")?;
    profile_to_edit.git_config.user_name = new_user_name;

    // User Email
    let new_user_email = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("User email")
        .default(profile_to_edit.git_config.user_email.clone())
        // Add basic email validation if dialoguer supports it, or rely on Profile::validate
        .interact_text()
        .context("Failed to get user email input.")?;
    profile_to_edit.git_config.user_email = new_user_email;

    // Git User Signing Key
    let new_signing_key_str = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Git User Signing Key (for commit signing, e.g., GPG key ID or SSH key path, leave blank for none)")
        .default(profile_to_edit.git_config.user_signingkey.clone().unwrap_or_default())
        .allow_empty(true)
        .interact_text()
        .context("Failed to get signing key input.")?;
    profile_to_edit.git_config.user_signingkey = if new_signing_key_str.is_empty() {
        None
    } else {
        Some(new_signing_key_str)
    };

    // SSH Key Path
    let new_ssh_key_str = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Path to SSH private key (leave blank for none)")
        .default(
            profile_to_edit
                .ssh_key
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default(),
        )
        .allow_empty(true)
        .interact_text()
        .context("Failed to get SSH key path input.")?;
    profile_to_edit.ssh_key = if new_ssh_key_str.is_empty() {
        None
    } else {
        Some(PathBuf::from(new_ssh_key_str))
    };

    // Associated GPG Key ID
    let new_gpg_key_str = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Associated GPG Key ID (optional, for other GPG uses, leave blank for none)")
        .default(profile_to_edit.gpg_key.clone().unwrap_or_default())
        .allow_empty(true)
        .interact_text()
        .context("Failed to get GPG key ID input.")?;
    profile_to_edit.gpg_key = if new_gpg_key_str.is_empty() {
        None
    } else {
        Some(new_gpg_key_str)
    };

    // Validate the modified profile
    if let Err(validation_error) = profile_to_edit.validate() {
        bail!(
            "Profile validation failed after edits: {}\nChanges not saved.",
            validation_error.to_string().red()
        );
    }

    config
        .save()
        .context("Failed to save configuration after editing profile.")?;

    println!("Profile '{}' updated successfully.", name.green());

    Ok(())
}

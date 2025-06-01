use anyhow::{bail, Context, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Input};
use std::path::PathBuf;

use crate::config::Config;

pub fn execute(
    name: String,
    cli_user_name: Option<String>,
    cli_user_email: Option<String>,
    cli_signing_key: Option<String>,
    cli_ssh_key_path: Option<String>,
    cli_gpg_key_id: Option<String>,
) -> Result<()> {
    let mut config = Config::load().context("Failed to load configuration.")?;

    let profile_to_edit = config
        .profiles
        .get_mut(&name)
        .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found.", name.cyan()))?;

    let is_non_interactive = cli_user_name.is_some()
        || cli_user_email.is_some()
        || cli_signing_key.is_some()
        || cli_ssh_key_path.is_some()
        || cli_gpg_key_id.is_some();

    if is_non_interactive {
        println!(
            "Editing profile '{}' non-interactively.",
            name.cyan().bold()
        );

        if let Some(uname) = cli_user_name {
            if uname.trim().is_empty() {
                bail!("User name cannot be set to empty in non-interactive mode.");
            }
            profile_to_edit.git_config.user_name = uname.trim().to_string();
            println!(
                "  Updated user name to: {}",
                profile_to_edit.git_config.user_name.green()
            );
        }

        if let Some(email) = cli_user_email {
            if email.trim().is_empty() {
                bail!("User email cannot be set to empty in non-interactive mode.");
            }
            profile_to_edit.git_config.user_email = email.trim().to_string();
            println!(
                "  Updated user email to: {}",
                profile_to_edit.git_config.user_email.green()
            );
        }

        if let Some(key) = cli_signing_key {
            if key.trim().is_empty() {
                profile_to_edit.git_config.user_signingkey = None;
                println!("  {} Git signing key.", "Removed".yellow());
            } else {
                profile_to_edit.git_config.user_signingkey = Some(key.trim().to_string());
                println!("  Updated Git signing key to: {}", key.trim().green());
            }
        }

        if let Some(path) = cli_ssh_key_path {
            if path.trim().is_empty() {
                profile_to_edit.ssh_key = None;
                println!("  {} SSH key path.", "Removed".yellow());
            } else {
                profile_to_edit.ssh_key = Some(PathBuf::from(path.trim()));
                println!("  Updated SSH key path to: {}", path.trim().green());
            }
        }

        if let Some(id) = cli_gpg_key_id {
            if id.trim().is_empty() {
                profile_to_edit.gpg_key = None;
                println!("  {} GPG key ID.", "Removed".yellow());
            } else {
                profile_to_edit.gpg_key = Some(id.trim().to_string());
                println!("  Updated GPG key ID to: {}", id.trim().green());
            }
        }
    } else {
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
        if new_user_name.trim().is_empty() {
            bail!("User name cannot be empty. Edit aborted.");
        }
        profile_to_edit.git_config.user_name = new_user_name.trim().to_string();

        // User Email
        let new_user_email = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("User email")
            .default(profile_to_edit.git_config.user_email.clone())
            .interact_text()
            .context("Failed to get user email input.")?;
        if new_user_email.trim().is_empty() {
            bail!("User email cannot be empty. Edit aborted.");
        }
        profile_to_edit.git_config.user_email = new_user_email.trim().to_string();

        // Git User Signing Key
        let new_signing_key_str = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Git User Signing Key (for commit signing, e.g., GPG key ID or SSH key path, leave blank for none)")
            .default(profile_to_edit.git_config.user_signingkey.clone().unwrap_or_default())
            .allow_empty(true)
            .interact_text()
            .context("Failed to get signing key input.")?;
        profile_to_edit.git_config.user_signingkey = if new_signing_key_str.trim().is_empty() {
            None
        } else {
            Some(new_signing_key_str.trim().to_string())
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
        profile_to_edit.ssh_key = if new_ssh_key_str.trim().is_empty() {
            None
        } else {
            Some(PathBuf::from(new_ssh_key_str.trim()))
        };

        // Associated GPG Key ID
        let new_gpg_key_str = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(
                "Associated GPG Key ID (optional, for other GPG uses, leave blank for none)",
            )
            .default(profile_to_edit.gpg_key.clone().unwrap_or_default())
            .allow_empty(true)
            .interact_text()
            .context("Failed to get GPG key ID input.")?;
        profile_to_edit.gpg_key = if new_gpg_key_str.trim().is_empty() {
            None
        } else {
            Some(new_gpg_key_str.trim().to_string())
        };
    }

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

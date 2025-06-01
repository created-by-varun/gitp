use anyhow::{bail, Context, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Input};

use crate::config::{Config, Profile, ValidationError};

pub fn execute(
    profile_name: String,
    cli_user_name: Option<String>,
    cli_user_email: Option<String>,
    cli_signing_key: Option<String>,
    cli_ssh_key_path: Option<String>,
    cli_gpg_key_id: Option<String>,
) -> Result<()> {
    let mut config = Config::load().context("Failed to load configuration. Ensure ~/.config/gitp/config.toml is accessible or run init if applicable.")?;

    if config.profiles.contains_key(&profile_name) {
        bail!(
            "Profile '{}' already exists. Choose a different name or edit the existing one.",
            profile_name.yellow()
        );
    }

    println!("Creating new profile: {}", profile_name.cyan().bold());

    let mut new_profile: Profile;

    let is_non_interactive = if let (Some(name), Some(email)) = (&cli_user_name, &cli_user_email) {
        !name.trim().is_empty() && !email.trim().is_empty()
    } else {
        false
    };

    if is_non_interactive {
        println!("Running in non-interactive mode (user_name and user_email provided).");
        // Guaranteed to have Some(non-empty) for name and email due to is_non_interactive check
        new_profile = Profile::new(
            profile_name.clone(),
            cli_user_name.as_ref().unwrap().trim().to_string(),
            cli_user_email.as_ref().unwrap().trim().to_string(),
        );

        if let Some(key) = &cli_signing_key {
            if !key.trim().is_empty() {
                new_profile.git_config.user_signingkey = Some(key.trim().to_string());
            }
        }
        if let Some(path) = &cli_ssh_key_path {
            if !path.trim().is_empty() {
                new_profile.ssh_key = Some(path.trim().into());
            }
        }
        if let Some(id) = &cli_gpg_key_id {
            if !id.trim().is_empty() {
                new_profile.gpg_key = Some(id.trim().to_string());
            }
        }
    } else {
        println!("Running in interactive mode.");
        let user_name_input: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter Git user name (e.g., John Doe)")
            .interact_text()
            .context("Failed to get user name input.")?;

        let user_email_input: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter Git user email (e.g., john.doe@example.com)")
            .interact_text()
            .context("Failed to get user email input.")?;

        if user_name_input.trim().is_empty() {
            bail!("User name cannot be empty. Profile creation aborted.");
        }
        if user_email_input.trim().is_empty() {
            bail!("User email cannot be empty. Profile creation aborted.");
        }

        new_profile = Profile::new(
            profile_name.clone(),
            user_name_input.trim().to_string(),
            user_email_input.trim().to_string(),
        );

        let signing_key_input: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter Git signing key (optional, press Enter to skip)")
            .allow_empty(true)
            .interact_text()
            .context("Failed to get signing key input.")?;
        if !signing_key_input.trim().is_empty() {
            new_profile.git_config.user_signingkey = Some(signing_key_input.trim().to_string());
        }

        let ssh_key_path_input: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter path to SSH key (optional, press Enter to skip)")
            .allow_empty(true)
            .interact_text()
            .context("Failed to get SSH key path input.")?;
        if !ssh_key_path_input.trim().is_empty() {
            new_profile.ssh_key = Some(ssh_key_path_input.trim().into());
        }

        let gpg_key_id_input: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter GPG key ID for signing (optional, press Enter to skip)")
            .allow_empty(true)
            .interact_text()
            .context("Failed to get GPG key ID input.")?;
        if !gpg_key_id_input.trim().is_empty() {
            new_profile.gpg_key = Some(gpg_key_id_input.trim().to_string());
        }
    }

    // Validate the newly created profile
    if let Err(validation_error) = new_profile.validate() {
        match validation_error {
            ValidationError::EmptyName => bail!("Profile name cannot be empty (internal validation). This should ideally be caught earlier."),
            ValidationError::EmptyUserName => bail!("User name cannot be empty (internal validation). This should be caught by prompt validation."),
            ValidationError::EmptyEmail => bail!("User email cannot be empty (internal validation). This should be caught by prompt validation."),
            ValidationError::InvalidEmail(email) => bail!("Invalid email format provided: '{}'. Please enter a valid email.", email.yellow()),
            ValidationError::SshKeyNotFound(path) => bail!("SSH key not found at path: '{}' (internal validation). This field is not prompted yet.", path.display()),
        }
    }

    config.profiles.insert(profile_name.clone(), new_profile);
    config.save().context(
        "Failed to save configuration. Check permissions for ~/.config/gitp/config.toml.",
    )?;

    println!("\nProfile '{}' created successfully!", profile_name.green());

    if !is_non_interactive {
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Do you want to use (activate) profile '{}' now?",
                profile_name.cyan()
            ))
            .default(true)
            .interact()?
        {
            // Directly call the use_profile execute function
            // Defaulting to global activation (local=false, global=true)
            match crate::commands::use_profile::execute(profile_name.clone(), false, true) {
                Ok(_) => println!("Profile '{}' activated globally.", profile_name.green()),
                Err(e) => eprintln!(
                    "Failed to activate profile '{}': {}",
                    profile_name.yellow(),
                    e.to_string().red()
                ),
            }
        } else {
            println!(
                "You can activate it later using: {}",
                format!("gitp use {}", profile_name).yellow()
            );
        }
    }

    Ok(())
}

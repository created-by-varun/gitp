use anyhow::{bail, Context, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Input};

use crate::config::{Config, Profile, ValidationError};

pub fn execute(profile_name: String, interactive: bool) -> Result<()> {
    let mut config = Config::load().context("Failed to load configuration. Ensure ~/.config/gitp/config.toml is accessible or run init if applicable.")?;

    if config.profiles.contains_key(&profile_name) {
        bail!("Profile '{}' already exists. Choose a different name or edit the existing one.", profile_name.yellow());
    }

    println!("Creating new profile: {}", profile_name.cyan().bold());

    let user_name_input: String;
    let user_email_input: String;

    if interactive {
        user_name_input = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter Git user name (e.g., John Doe)")
            .interact_text()
            .context("Failed to get user name input. Please try again.")?;

        user_email_input = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter Git user email (e.g., john.doe@example.com)")
            .interact_text()
            .context("Failed to get user email input. Please try again.")?;

        if user_name_input.trim().is_empty() {
            bail!("User name cannot be empty. Profile creation aborted.");
        }
        if user_email_input.trim().is_empty() {
            bail!("User email cannot be empty. Profile creation aborted.");
        }
    } else {
        bail!("Non-interactive profile creation currently requires user name and email to be provided via CLI flags (e.g., --git-name, --email). These flags are not yet implemented. Please use interactive mode (-i) for now.");
    }

    let new_profile = Profile::new(
        profile_name.clone(),
        user_name_input,
        user_email_input,
    );

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
    config.save().context("Failed to save configuration. Check permissions for ~/.config/gitp/config.toml.")?;

    println!("\nProfile '{}' created successfully!", profile_name.green());

    if interactive {
        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Do you want to use (activate) profile '{}' now?", profile_name.cyan()))
            .default(true)
            .interact()? 
        {
            // For now, just print the command. Direct execution can be added later.
            println!("To use this profile, run: {}", format!("gitp use {}", profile_name).yellow());
            println!("Note: Full activation logic for 'gitp use' is pending implementation.");
        } else {
            println!("You can activate it later using: {}", format!("gitp use {}", profile_name).yellow());
        }
    }

    Ok(())
}

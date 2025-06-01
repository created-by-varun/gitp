use anyhow::{bail, Context, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm};

use crate::config::{Config, CredentialType};
use crate::credentials::keyring::delete_token;

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

    // Get the profile before removing it to check for keychain credentials
    let profile_to_remove = config.profiles.get(&name).cloned(); // Cloned to avoid borrow issues

    // Remove the profile from the HashMap
    if config.profiles.remove(&name).is_some() {
        if let Some(profile) = profile_to_remove {
            if let Some(https_creds) = profile.https_credentials {
                if let CredentialType::KeychainRef(keychain_username) = https_creds.credential_type
                {
                    match delete_token(&https_creds.host, &keychain_username) {
                        Ok(_) => println!(
                            "  Successfully deleted token for {}@{} from keychain.",
                            keychain_username.cyan(),
                            https_creds.host.green()
                        ),
                        Err(e) => eprintln!(
                            "  {}: Failed to delete token for {}@{} from keychain: {}. Please remove it manually if needed.",
                            "Warning".yellow(),
                            keychain_username.cyan(),
                            https_creds.host.green(),
                            e
                        ),
                    }
                }
            }
        }
    } else {
        // This case should ideally not be reached if the initial check (line 9) passes
        bail!(
            "Profile '{}' was expected but not found during removal operation.",
            name.yellow()
        );
    }

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

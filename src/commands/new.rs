use anyhow::{bail, Context, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password};

use crate::config::{Config, CredentialType, HttpsCredentials, Profile, ValidationError};

pub fn execute(
    profile_name: String,
    cli_user_name: Option<String>,
    cli_user_email: Option<String>,
    cli_signing_key: Option<String>,
    cli_ssh_key_path: Option<String>,
    cli_gpg_key_id: Option<String>,
    cli_https_host: Option<String>,
    cli_https_username: Option<String>,
    cli_https_token: Option<String>,
    cli_https_store_in_keychain: bool, // Updated argument
    cli_ssh_key_host: Option<String>,
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
                // If SSH key path is provided, check for SSH key host
                if let Some(host) = &cli_ssh_key_host {
                    if !host.trim().is_empty() {
                        new_profile.ssh_key_host = Some(host.trim().to_string());
                    }
                }
            }
        }
        if let Some(id) = &cli_gpg_key_id {
            if !id.trim().is_empty() {
                new_profile.gpg_key = Some(id.trim().to_string());
            }
        }

        // Handle HTTPS credentials in non-interactive mode
        if let (Some(host_str), Some(username_str), Some(token_str)) =
            (&cli_https_host, &cli_https_username, &cli_https_token)
        {
            if !host_str.trim().is_empty()
                && !username_str.trim().is_empty()
                && !token_str.trim().is_empty()
            {
                let host = host_str.trim().to_string();
                let username = username_str.trim().to_string();
                let token = token_str.trim().to_string();

                let credential_type = if cli_https_store_in_keychain {
                    match crate::credentials::keyring::store_token(&host, &username, &token) {
                        Ok(_) => {
                            println!(
                                "  Stored HTTPS token for {}@{} in keychain.",
                                username.cyan(),
                                host.green()
                            );
                            CredentialType::KeychainRef(username.clone())
                        }
                        Err(e) => {
                            eprintln!(
                                "  {}: Failed to store HTTPS token in keychain for {}@{}: {}. Storing as plain text instead.",
                                "Warning".yellow(),
                                username.cyan(),
                                host.green(),
                                e
                            );
                            CredentialType::Token(token)
                        }
                    }
                } else {
                    CredentialType::Token(token)
                };

                new_profile.https_credentials = Some(HttpsCredentials {
                    host,
                    username,
                    credential_type,
                });
                println!(
                    "  Configured HTTPS credentials for host: {}",
                    host_str.trim().green()
                );
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

            let ssh_key_host_input: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter SSH key host (e.g., github.com, gitlab.mycompany.com)")
                .allow_empty(false) // Host cannot be empty if key is provided
                .interact_text()
                .context("Failed to get SSH key host input.")?;
            if !ssh_key_host_input.trim().is_empty() {
                // Redundant check due to allow_empty(false), but good practice
                new_profile.ssh_key_host = Some(ssh_key_host_input.trim().to_string());
            }
        }

        let gpg_key_id_input: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter GPG key ID for signing (optional, press Enter to skip)")
            .allow_empty(true)
            .interact_text()
            .context("Failed to get GPG key ID input.")?;
        if !gpg_key_id_input.trim().is_empty() {
            new_profile.gpg_key = Some(gpg_key_id_input.trim().to_string());
        }

        // HTTPS Credentials Interactive Prompts
        println!("\n{}", "HTTPS Credentials (optional):".cyan());
        let https_host_input: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter HTTPS host (e.g., github.com, leave blank to skip)")
            .allow_empty(true)
            .interact_text()
            .context("Failed to get HTTPS host input.")?;

        if !https_host_input.trim().is_empty() {
            let https_username_input: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt(format!(
                    "Enter HTTPS username for host '{}'",
                    https_host_input.trim()
                ))
                .interact_text()
                .context("Failed to get HTTPS username input.")?;

            if https_username_input.trim().is_empty() {
                bail!("HTTPS username cannot be empty if host is provided. HTTPS credentials setup aborted.");
            }

            let token_input: String = Password::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter HTTPS Token")
                .with_confirmation("Confirm HTTPS Token", "Tokens do not match.")
                .interact()
                .context("Failed to get HTTPS token input.")?;
            if token_input.trim().is_empty() {
                bail!("Token cannot be empty. HTTPS credentials setup aborted.");
            }

            let credential_type_value = if Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Store this HTTPS token securely in the system keychain?")
                .default(true)
                .interact()?
            {
                match crate::credentials::keyring::store_token(
                    https_host_input.trim(),
                    https_username_input.trim(),
                    token_input.trim(),
                ) {
                    Ok(_) => {
                        println!(
                            "  Stored HTTPS token for {}@{} in keychain.",
                            https_username_input.trim().cyan(),
                            https_host_input.trim().green()
                        );
                        CredentialType::KeychainRef(https_username_input.trim().to_string())
                    }
                    Err(e) => {
                        eprintln!(
                            "  {}: Failed to store HTTPS token in keychain: {}. Storing as plain text instead.",
                            "Warning".yellow(),
                            e
                        );
                        CredentialType::Token(token_input.trim().to_string())
                    }
                }
            } else {
                CredentialType::Token(token_input.trim().to_string())
            };

            new_profile.https_credentials = Some(HttpsCredentials {
                host: https_host_input.trim().to_string(),
                username: https_username_input.trim().to_string(),
                credential_type: credential_type_value,
            });
        }
    }

    // Validate the newly created profile
    if let Err(validation_error) = new_profile.validate() {
        let error_message = match validation_error {
            ValidationError::EmptyName => "Profile name cannot be empty.".to_string(),
            ValidationError::EmptyUserName => "User name cannot be empty.".to_string(),
            ValidationError::EmptyEmail => "User email cannot be empty.".to_string(),
            ValidationError::InvalidEmail(email) => format!("Invalid email format: '{}'.", email),
            ValidationError::SshKeyNotFound(path) => {
                format!("SSH key not found: '{}'.", path.display())
            }
            ValidationError::InvalidGpgKeyFormat(key) => {
                format!(
                    "Invalid GPG key format for '{}'. Expected 8, 16, or 40 hex characters.",
                    key
                )
            }
            ValidationError::EmptySshKeyHost => {
                "SSH key host cannot be empty when an SSH key is provided.".to_string()
            }
            ValidationError::EmptyHttpsHost => {
                "HTTPS credentials host cannot be empty.".to_string()
            }
            ValidationError::EmptyHttpsUsername => {
                "HTTPS credentials username cannot be empty.".to_string()
            }
            ValidationError::EmptyHttpsToken => {
                "HTTPS credentials token cannot be empty when type is Token.".to_string()
            }
            ValidationError::EmptyHttpsKeychainRef => {
                "HTTPS credentials keychain reference cannot be empty when type is KeychainRef."
                    .to_string()
            }
        };
        bail!(error_message);
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

use anyhow::{bail, Context, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password};
use std::path::PathBuf;

use crate::config::{Config, CredentialType, HttpsCredentials};

pub fn execute(
    name: String,
    cli_user_name: Option<String>,
    cli_user_email: Option<String>,
    cli_signing_key: Option<String>,
    cli_ssh_key_path: Option<String>,
    cli_gpg_key_id: Option<String>,
    cli_https_host: Option<String>,
    cli_https_username: Option<String>,
    cli_https_token: Option<String>,
    cli_https_keychain_ref: Option<String>,
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
        || cli_gpg_key_id.is_some()
        || cli_https_host.is_some()
        || cli_https_username.is_some()
        || cli_https_token.is_some()
        || cli_https_keychain_ref.is_some();

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

        // Handle HTTPS credentials in non-interactive mode
        if let Some(host_cli_val) = &cli_https_host {
            if host_cli_val.trim().is_empty() {
                // Remove HTTPS credentials if host is provided as empty string
                if profile_to_edit.https_credentials.is_some() {
                    profile_to_edit.https_credentials = None;
                    println!("  {} HTTPS credentials.", "Removed".yellow());
                }
            } else {
                // Host is provided and not empty, username must also be provided (enforced by clap)
                let username_cli_val = cli_https_username
                    .as_ref()
                    .expect("--https-username is required when --https-host is provided");
                if username_cli_val.trim().is_empty() {
                    bail!("HTTPS username cannot be set to empty in non-interactive mode when host is provided.");
                }

                let mut current_https_creds = profile_to_edit
                    .https_credentials
                    .take()
                    .unwrap_or_else(|| HttpsCredentials {
                        host: String::new(),
                        username: String::new(),
                        credential_type: CredentialType::Token(String::new()), // Placeholder, will be overwritten or removed
                    });

                current_https_creds.host = host_cli_val.trim().to_string();
                current_https_creds.username = username_cli_val.trim().to_string();
                println!(
                    "  Updated HTTPS host to: {}",
                    current_https_creds.host.green()
                );
                println!(
                    "  Updated HTTPS username to: {}",
                    current_https_creds.username.green()
                );

                let mut cred_updated = false;
                if let Some(token_cli_val) = &cli_https_token {
                    if token_cli_val.trim().is_empty() {
                        // This case might be tricky if user wants to remove token but keep host/user.
                        // For now, if token is empty string, we assume it means no specific update to token type from CLI.
                        // If they want to remove token, they should remove the whole https_credential block by setting host to ""
                        println!("  HTTPS token provided as empty, no change to credential type based on token.");
                    } else {
                        current_https_creds.credential_type =
                            CredentialType::Token(token_cli_val.trim().to_string());
                        println!("  Updated HTTPS credential to use Token.");
                        cred_updated = true;
                    }
                } else if let Some(keychain_ref_cli_val) = &cli_https_keychain_ref {
                    if keychain_ref_cli_val.trim().is_empty() {
                        println!("  HTTPS keychain reference provided as empty, no change to credential type based on keychain ref.");
                    } else {
                        current_https_creds.credential_type =
                            CredentialType::KeychainRef(keychain_ref_cli_val.trim().to_string());
                        println!("  Updated HTTPS credential to use Keychain Reference.");
                        cred_updated = true;
                    }
                }

                // If neither token nor keychain_ref was provided via CLI but we had existing creds,
                // ensure the credential_type is still valid or handle it.
                // For now, if only host/username are changed, the existing credential_type is preserved.
                // If no cred_updated and the placeholder Token(String::new()) is still there from a new HttpsCredentials, this is an invalid state.
                // However, clap's 'requires_all' for token/keychain_ref on New command and similar logic should prevent this.
                // For Edit, if user provides only host and username, we preserve existing credential type.
                // If no existing credential type, and no new one provided, this is an issue if we just created HttpsCredentials.
                // This logic assumes that if HttpsCredentials struct exists, its credential_type is valid.
                if !cred_updated
                    && matches!(current_https_creds.credential_type, CredentialType::Token(ref t) if t.is_empty())
                    && profile_to_edit.https_credentials.is_none()
                {
                    // This means we created a new HttpsCredentials with a placeholder empty token, and no new token/keychain was given.
                    // This state should ideally be prevented by CLI arg validation or a more robust state machine here.
                    // For now, we'll remove it to avoid saving invalid state.
                    println!("  HTTPS host and username provided, but no credential detail. Removing HTTPS credentials.");
                } else {
                    profile_to_edit.https_credentials = Some(current_https_creds);
                }
            }
        }
    } else {
        println!("Editing profile: {}", name.cyan().bold());
        println!("{}", "(Press Enter to keep current value, if any)".dimmed());
        // HTTPS Credentials Interactive Editing
        println!();
        println!("{}", "HTTPS Credentials Configuration:".bold());

        let current_https_creds = profile_to_edit.https_credentials.clone();
        if let Some(creds) = &current_https_creds {
            println!("  Current host: {}", creds.host.yellow());
            println!("  Current username: {}", creds.username.yellow());
            match &creds.credential_type {
                CredentialType::Token(_) => {
                    println!("  Current type: {}", "Token (value is masked)".yellow())
                }
                CredentialType::KeychainRef(r) => {
                    println!("  Current type: Keychain Reference ({})", r.yellow())
                }
            }
        } else {
            println!("  {}", "No HTTPS credentials currently set.".dimmed());
        }

        if Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to configure or update HTTPS credentials?")
            .default(current_https_creds.is_some()) // Default to yes if creds exist, no otherwise
            .interact()?
        {
            let https_host_input: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt(
                    "HTTPS Host (e.g., github.com, leave blank to remove if currently set)",
                )
                .default(
                    current_https_creds
                        .as_ref()
                        .map_or_else(String::new, |c| c.host.clone()),
                )
                .allow_empty(true)
                .interact_text()
                .context("Failed to get HTTPS host input.")?;

            if https_host_input.trim().is_empty() {
                if profile_to_edit.https_credentials.is_some() {
                    profile_to_edit.https_credentials = None;
                    println!("  {}", "HTTPS credentials removed.".yellow());
                }
            } else {
                let https_username_input: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("HTTPS Username")
                    .default(
                        current_https_creds
                            .as_ref()
                            .filter(|c| c.host == https_host_input.trim())
                            .map_or_else(String::new, |c| c.username.clone()),
                    )
                    .interact_text()
                    .context("Failed to get HTTPS username input.")?;

                if https_username_input.trim().is_empty() {
                    bail!("HTTPS username cannot be empty if host is provided. HTTPS credentials setup aborted.");
                }

                let credential_choices =
                    &["Personal Access Token (PAT)", "System Keychain Reference"];
                let current_type_idx =
                    current_https_creds
                        .as_ref()
                        .map_or(0, |c| match c.credential_type {
                            CredentialType::Token(_) => 0,
                            CredentialType::KeychainRef(_) => 1,
                        });

                let credential_selection: usize =
                    dialoguer::Select::with_theme(&ColorfulTheme::default())
                        .with_prompt("Choose HTTPS credential type")
                        .items(credential_choices)
                        .default(current_type_idx)
                        .interact()
                        .context("Failed to get credential type choice.")?;

                let credential_type_value = match credential_selection {
                    0 => {
                        // Token
                        let token_input: String = Password::with_theme(&ColorfulTheme::default())
                            .with_prompt("Enter Personal Access Token")
                            .interact()
                            .context("Failed to get token input.")?;
                        if token_input.trim().is_empty() {
                            bail!("Token cannot be empty. HTTPS credentials setup aborted.");
                        }
                        CredentialType::Token(token_input.trim().to_string())
                    }
                    1 => {
                        // KeychainRef
                        let keychain_ref_input: String =
                            Input::with_theme(&ColorfulTheme::default())
                                .with_prompt("Enter Keychain Reference string")
                                .interact_text()
                                .context("Failed to get keychain reference input.")?;
                        if keychain_ref_input.trim().is_empty() {
                            bail!("Keychain reference cannot be empty. HTTPS credentials setup aborted.");
                        }
                        CredentialType::KeychainRef(keychain_ref_input.trim().to_string())
                    }
                    _ => unreachable!(), // Should not happen with Select
                };

                profile_to_edit.https_credentials = Some(HttpsCredentials {
                    host: https_host_input.trim().to_string(),
                    username: https_username_input.trim().to_string(),
                    credential_type: credential_type_value,
                });
                println!("  {}", "HTTPS credentials configured.".green());
            }
        } else if profile_to_edit.https_credentials.is_some() {
            // User chose not to configure/update, but creds exist
            if !Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Keep existing HTTPS credentials?")
                .default(true)
                .interact()?
            {
                profile_to_edit.https_credentials = None;
                println!(
                    "  {}",
                    "Existing HTTPS credentials removed as per choice.".yellow()
                );
            }
        }
        println!(); // Add a blank line after HTTPS config section

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
        let error_message = match validation_error {
            crate::config::ValidationError::EmptyName => "Profile name cannot be empty.".to_string(),
            crate::config::ValidationError::EmptyUserName => "User name cannot be empty.".to_string(),
            crate::config::ValidationError::EmptyEmail => "User email cannot be empty.".to_string(),
            crate::config::ValidationError::InvalidEmail(email) => format!("Invalid email format: '{}'.", email),
            crate::config::ValidationError::SshKeyNotFound(path) => format!("SSH key not found: '{}'.", path.display()),
            crate::config::ValidationError::InvalidGpgKeyFormat(key) => {
                format!("Invalid GPG key format for '{}'. Expected 8, 16, or 40 hex characters.", key)
            }
            crate::config::ValidationError::EmptyHttpsHost => "HTTPS credentials host cannot be empty.".to_string(),
            crate::config::ValidationError::EmptyHttpsUsername => "HTTPS credentials username cannot be empty.".to_string(),
            crate::config::ValidationError::EmptyHttpsToken => "HTTPS credentials token cannot be empty when type is Token.".to_string(),
            crate::config::ValidationError::EmptyHttpsKeychainRef => "HTTPS credentials keychain reference cannot be empty when type is KeychainRef.".to_string(),
        };
        bail!(
            "Profile validation failed after edits: {}\nChanges not saved.",
            error_message.red()
        );
    }

    config
        .save()
        .context("Failed to save configuration after editing profile.")?;

    println!("Profile '{}' updated successfully.", name.green());

    Ok(())
}

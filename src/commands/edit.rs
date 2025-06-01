use anyhow::{bail, Context, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password};
use std::path::PathBuf;

use crate::config::{Config, CredentialType, HttpsCredentials};
use crate::credentials::keyring::{delete_token, store_token}; // Added keyring imports

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
    // cli_https_keychain_ref: Option<String>, // Removed
    cli_https_store_in_keychain: bool,
    cli_https_remove_credentials: bool,
    cli_ssh_key_host: Option<String>,
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
        // || cli_https_keychain_ref.is_some() // Removed
        || cli_https_store_in_keychain // This is a bool, presence means non-interactive intent if other flags are set or if it's true
        || cli_https_remove_credentials // Same for this flag
        || cli_ssh_key_host.is_some();

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
                profile_to_edit.ssh_key_host = None; // Clear host if key path is cleared
                println!("  {} SSH key path and host.", "Removed".yellow());
            } else {
                profile_to_edit.ssh_key = Some(PathBuf::from(path.trim()));
                println!("  Updated SSH key path to: {}", path.trim().green());
                // Handle ssh_key_host only if ssh_key_path was provided
                if let Some(host) = cli_ssh_key_host.as_deref() {
                    // Use as_deref to work with &str
                    if host.trim().is_empty() {
                        profile_to_edit.ssh_key_host = None;
                        println!("  {} SSH key host.", "Removed".yellow());
                    } else {
                        profile_to_edit.ssh_key_host = Some(host.trim().to_string());
                        println!("  Updated SSH key host to: {}", host.trim().green());
                    }
                } else if profile_to_edit.ssh_key.is_some()
                    && profile_to_edit.ssh_key_host.is_none()
                {
                    // If ssh_key_path was just set and no host was provided via CLI, but one might be needed.
                    // This case is tricky for non-interactive. Validation will catch it if host is required.
                    // Or, we might decide CLI must provide host if path is given and new.
                }
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
        if cli_https_remove_credentials {
            if let Some(existing_creds) = profile_to_edit.https_credentials.take() {
                // Use take to remove it
                if let CredentialType::KeychainRef(keychain_username) =
                    existing_creds.credential_type
                {
                    match delete_token(&existing_creds.host, &keychain_username) {
                        Ok(_) => println!(
                            "  Successfully deleted token for {}@{} from keychain.",
                            keychain_username.cyan(),
                            existing_creds.host.green()
                        ),
                        Err(e) => eprintln!(
                            "  {}: Failed to delete token for {}@{} from keychain: {}. Please remove it manually if needed.",
                            "Warning".yellow(),
                            keychain_username.cyan(),
                            existing_creds.host.green(),
                            e
                        ),
                    }
                }
                println!(
                    "  {} HTTPS credentials for host '{}'.",
                    "Removed".yellow(),
                    existing_creds.host.green()
                );
            } else {
                println!(
                    "  No HTTPS credentials found for profile '{}' to remove.",
                    name.cyan()
                );
            }
        } else if let Some(host_cli_val) = &cli_https_host {
            // This block executes if --https-remove-credentials is false
            // AND --https-host is Some (implies trying to set/update credentials).
            // clap rules should ensure that if --https-host is Some, then --https-username is also Some.

            let new_host_untrimmed = host_cli_val; // host_cli_val is &String from Option<&String>
            let new_host = new_host_untrimmed.trim().to_string();

            if new_host.is_empty() {
                // This case should ideally not be reached if --https-remove-credentials is the way to remove.
                // Or if clap validates that --https-host cannot be empty if provided for an update.
                // For robustness, treat as a warning and no-op for HTTPS credentials.
                eprintln!(
                    "  {}: --https-host was provided as empty when not using --https-remove-credentials. No changes made to HTTPS credentials.",
                    "Warning".yellow()
                );
            } else {
                // Host is not empty. Username must be present (clap: requires = "https_host" on https_username).
                let new_username = cli_https_username
                    .as_ref()
                    .map(|s| s.trim().to_string())
                    .expect("--https-username is required by clap if --https-host is provided"); // Should be guaranteed by clap
                if new_username.is_empty() {
                    bail!("HTTPS username cannot be empty when --https-host is provided.");
                }

                // If --https-token is provided, we proceed to update/set credentials.
                if let Some(new_token_val) = &cli_https_token {
                    let new_token = new_token_val.trim().to_string();
                    if new_token.is_empty() {
                        bail!("HTTPS token cannot be set to empty in non-interactive mode. Use --https-remove-credentials to remove all HTTPS credentials, or provide a valid token.");
                    }

                    // Check if existing credentials need keychain cleanup
                    let mut old_keychain_creds_to_delete: Option<(String, String)> = None;
                    if let Some(ref existing_creds) = profile_to_edit.https_credentials {
                        if let CredentialType::KeychainRef(ref old_keychain_username) =
                            existing_creds.credential_type
                        {
                            // Conditions for deleting old keychain entry:
                            // 1. Host is changing.
                            // 2. Username (keychain service user) is changing for the same host.
                            // 3. Host and username are the same, but user wants to switch from keychain to plain token.
                            if existing_creds.host != new_host
                                || (existing_creds.host == new_host
                                    && old_keychain_username != &new_username)
                                || (existing_creds.host == new_host
                                    && old_keychain_username == &new_username
                                    && !cli_https_store_in_keychain)
                            {
                                old_keychain_creds_to_delete = Some((
                                    existing_creds.host.clone(),
                                    old_keychain_username.clone(),
                                ));
                            }
                        }
                    }

                    if let Some((old_h, old_u)) = old_keychain_creds_to_delete {
                        match delete_token(&old_h, &old_u) {
                            Ok(_) => println!(
                                "  Successfully deleted previous token for {}@{} from keychain.",
                                old_u.cyan(),
                                old_h.green()
                            ),
                            Err(e) => eprintln!(
                                "  {}: Failed to delete previous token for {}@{} from keychain: {}. Please check manually.",
                                "Warning".yellow(),
                                old_u.cyan(),
                                old_h.green(),
                                e
                            ),
                        }
                    }

                    let final_credential_type;
                    if cli_https_store_in_keychain {
                        match store_token(&new_host, &new_username, &new_token) {
                            Ok(_) => {
                                final_credential_type =
                                    CredentialType::KeychainRef(new_username.clone());
                                println!(
                                    "  Successfully stored HTTPS token for {}@{} in keychain.",
                                    new_username.cyan(),
                                    new_host.green()
                                );
                            }
                            Err(e) => {
                                eprintln!(
                                    "  {}: Failed to store token in keychain: {}. Falling back to plain text storage in config.",
                                    "Warning".yellow(),
                                    e
                                );
                                final_credential_type = CredentialType::Token(new_token.clone());
                            }
                        }
                    } else {
                        final_credential_type = CredentialType::Token(new_token.clone());
                        println!(
                            "  Set HTTPS token for {}@{} (stored in config file).",
                            new_username.cyan(),
                            new_host.green()
                        );
                    }

                    profile_to_edit.https_credentials = Some(HttpsCredentials {
                        host: new_host.clone(),
                        username: new_username.clone(),
                        credential_type: final_credential_type,
                    });
                    println!("  Updated HTTPS credentials for profile '{}'.", name.cyan());
                } else {
                    // --https-host and --https-username provided, but --https-token is None.
                    // This means the user is trying to change host/username without providing a new token.
                    // This scenario is complex: what happens to the old token/keychain_ref?
                    // For simplicity in non-interactive, we will not modify existing token/keychain_ref if only host/user are changed without a new token.
                    // The old credentials (if any) will remain associated with their original host/user.
                    // A new entry is not created. User should use --https-remove-credentials and then add new ones, or provide all three.
                    println!(
                        "  {}: --https-host and --https-username provided without --https-token. ",
                        "Info".blue()
                    );
                    println!("  To set or update a token, please provide --https-host, --https-username, and --https-token together.");
                    println!("  No changes made to HTTPS credentials based on host/username alone without a token.");
                }
            }
        } else {
            // No HTTPS related flags were provided for an update (and not removing either)
            // This means no changes to HTTPS credentials in this non-interactive run.
            // This branch is needed to ensure the if/else if chain has a fallthrough for the Result type if other non-interactive flags were set.
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
                if let Some(ref actual_current_creds) = current_https_creds {
                    // Use the cloned current_https_creds
                    if let CredentialType::KeychainRef(ref keychain_username_to_delete) =
                        actual_current_creds.credential_type
                    {
                        match delete_token(&actual_current_creds.host, keychain_username_to_delete) {
                            Ok(_) => println!(
                                "  Successfully deleted token for {}@{} from keychain.",
                                keychain_username_to_delete.cyan(),
                                actual_current_creds.host.green()
                            ),
                            Err(e) => eprintln!(
                                "  {}: Failed to delete token for {}@{} from keychain: {}. Please remove it manually if needed.",
                                "Warning".yellow(),
                                keychain_username_to_delete.cyan(),
                                actual_current_creds.host.green(),
                                e
                            ),
                        }
                    }
                    profile_to_edit.https_credentials = None;
                    println!("  {}", "HTTPS credentials removed.".yellow());
                } else {
                    // No current credentials to remove, so do nothing.
                    println!("  No HTTPS credentials were set to remove.");
                }
            } else {
                let new_host = https_host_input.trim().to_string();
                let new_username: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("HTTPS Username")
                    .default(
                        current_https_creds
                            .as_ref()
                            .filter(|c| c.host == new_host)
                            .map_or_else(String::new, |c| c.username.clone()),
                    )
                    .interact_text()
                    .context("Failed to get HTTPS username input.")?;

                if new_username.trim().is_empty() {
                    bail!("HTTPS username cannot be empty if host is provided. HTTPS credentials setup aborted.");
                }
                let actual_new_username = new_username.trim().to_string();

                let store_in_keychain = Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Store this HTTPS token securely in the system keychain?")
                    .default(true)
                    .interact()?;

                let new_token: String = Password::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter Personal Access Token")
                    .interact()
                    .context("Failed to get token input.")?;
                if new_token.trim().is_empty() {
                    bail!("Token cannot be empty. HTTPS credentials setup aborted.");
                }
                let actual_new_token = new_token.trim().to_string();

                // Delete old keychain entry if necessary (before setting new credentials)
                if let Some(ref old_creds) = current_https_creds {
                    if let CredentialType::KeychainRef(ref old_keychain_username) =
                        old_creds.credential_type
                    {
                        let changing_host = old_creds.host != new_host;
                        let changing_username = old_keychain_username != &actual_new_username;
                        let switching_to_plain_text = !store_in_keychain;

                        if changing_host
                            || (old_creds.host == new_host && changing_username)
                            || (old_creds.host == new_host
                                && old_keychain_username == &actual_new_username
                                && switching_to_plain_text)
                        {
                            match delete_token(&old_creds.host, old_keychain_username) {
                                Ok(_) => println!(
                                    "  Successfully deleted previous token for {}@{} from keychain.",
                                    old_keychain_username.cyan(),
                                    old_creds.host.green()
                                ),
                                Err(e) => eprintln!(
                                    "  {}: Failed to delete previous token for {}@{} from keychain: {}. Please check manually.",
                                    "Warning".yellow(),
                                    old_keychain_username.cyan(),
                                    old_creds.host.green(),
                                    e
                                ),
                            }
                        }
                    }
                }

                let final_credential_type;
                if store_in_keychain {
                    match store_token(&new_host, &actual_new_username, &actual_new_token) {
                        Ok(_) => {
                            final_credential_type =
                                CredentialType::KeychainRef(actual_new_username.clone());
                            println!(
                                "  Successfully stored HTTPS token for {}@{} in keychain.",
                                actual_new_username.cyan(),
                                new_host.green()
                            );
                        }
                        Err(e) => {
                            eprintln!(
                                "  {}: Failed to store token in keychain: {}. Falling back to plain text storage in config.",
                                "Warning".yellow(),
                                e
                            );
                            final_credential_type = CredentialType::Token(actual_new_token.clone());
                        }
                    }
                } else {
                    final_credential_type = CredentialType::Token(actual_new_token.clone());
                    println!(
                        "  Set HTTPS token for {}@{} (stored in config file).",
                        actual_new_username.cyan(),
                        new_host.green()
                    );
                }

                profile_to_edit.https_credentials = Some(HttpsCredentials {
                    host: new_host,
                    username: actual_new_username,
                    credential_type: final_credential_type,
                });
                println!("  HTTPS credentials updated.");
            }
        } else if profile_to_edit.https_credentials.is_some() {
            // User chose not to configure/update, but creds exist
            if !Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Keep existing HTTPS credentials?")
                .default(true)
                .interact()?
            {
                // User chose to remove existing credentials
                if let Some(ref actual_current_creds) = current_https_creds {
                    // Use the cloned current_https_creds
                    if let CredentialType::KeychainRef(ref keychain_username_to_delete) =
                        actual_current_creds.credential_type
                    {
                        match delete_token(&actual_current_creds.host, keychain_username_to_delete) {
                            Ok(_) => println!(
                                "  Successfully deleted token for {}@{} from keychain.",
                                keychain_username_to_delete.cyan(),
                                actual_current_creds.host.green()
                            ),
                            Err(e) => eprintln!(
                                "  {}: Failed to delete token for {}@{} from keychain: {}. Please remove it manually if needed.",
                                "Warning".yellow(),
                                keychain_username_to_delete.cyan(),
                                actual_current_creds.host.green(),
                                e
                            ),
                        }
                    }
                }
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
        if new_ssh_key_str.trim().is_empty() {
            profile_to_edit.ssh_key = None;
            profile_to_edit.ssh_key_host = None; // Clear host if key path is cleared
        } else {
            profile_to_edit.ssh_key = Some(PathBuf::from(new_ssh_key_str.trim()));
            // If a new SSH key path is set, prompt for the host
            let new_ssh_key_host_str = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter SSH key host (e.g., github.com, required if SSH key is set)")
                .default(profile_to_edit.ssh_key_host.clone().unwrap_or_default())
                .allow_empty(false) // Host cannot be empty if key is provided
                .interact_text()
                .context("Failed to get SSH key host input.")?;
            if new_ssh_key_host_str.trim().is_empty() {
                // Should not happen due to allow_empty(false)
                // This case implies an issue or a desire to clear, but validation will prevent empty if key is set.
                // For safety, if somehow empty, treat as wanting to clear, though validation should catch this logic error.
                profile_to_edit.ssh_key_host = None;
            } else {
                profile_to_edit.ssh_key_host = Some(new_ssh_key_host_str.trim().to_string());
            }
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
            crate::config::ValidationError::EmptyName => {
                "Profile name cannot be empty.".to_string()
            }
            crate::config::ValidationError::EmptyUserName => {
                "User name cannot be empty.".to_string()
            }
            crate::config::ValidationError::EmptyEmail => "User email cannot be empty.".to_string(),
            crate::config::ValidationError::InvalidEmail(email) => {
                format!("Invalid email format: '{}'.", email)
            }
            crate::config::ValidationError::SshKeyNotFound(path) => {
                format!("SSH key not found: '{}'.", path.display())
            }
            crate::config::ValidationError::InvalidGpgKeyFormat(key) => {
                format!(
                    "Invalid GPG key format for '{}'. Expected 8, 16, or 40 hex characters.",
                    key
                )
            }
            crate::config::ValidationError::EmptySshKeyHost => {
                "SSH key host cannot be empty when an SSH key is provided.".to_string()
            }
            crate::config::ValidationError::EmptyHttpsHost => {
                "HTTPS credentials host cannot be empty.".to_string()
            }
            crate::config::ValidationError::EmptyHttpsUsername => {
                "HTTPS credentials username cannot be empty.".to_string()
            }
            crate::config::ValidationError::EmptyHttpsToken => {
                "HTTPS credentials token cannot be empty when type is Token.".to_string()
            }
            crate::config::ValidationError::EmptyHttpsKeychainRef => {
                "HTTPS credentials keychain reference cannot be empty when type is KeychainRef."
                    .to_string()
            }
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

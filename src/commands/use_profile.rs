// src/commands/use_profile.rs
use anyhow::{bail, Context, Result};
use colored::Colorize;

use crate::config::Config;
use crate::git::{set_git_config, unset_git_config, GitConfigScope};

pub fn execute(name: String, local: bool, global: bool) -> Result<()> {
    let mut config = Config::load().context("Failed to load configuration.")?;

    let profile_to_apply = config.profiles.get(&name).ok_or_else(|| {
        anyhow::anyhow!(
            "Profile '{}' not found. Use '{}' to list available profiles or '{}' to create a new one.",
            name.yellow(),
            "gitp list".cyan(),
            format!("gitp new {}", name).cyan()
        )
    })?;

    // Determine scope
    let scope = match (local, global) {
        (true, false) => GitConfigScope::Local,
        (false, true) => GitConfigScope::Global,
        (false, false) => GitConfigScope::Global, // Default to global
        (true, true) => {
            // This case should ideally be prevented by clap's arg parsing (e.g., mutually_exclusive_group)
            bail!("Cannot apply profile both locally and globally at the same time. Please specify only one.");
        }
    };

    let scope_str = format!("{:?}", scope).to_lowercase();

    println!(
        "Applying profile '{}' to {} Git configuration...",
        name.cyan(),
        scope_str
    );

    // Apply Git configurations
    set_git_config("user.name", &profile_to_apply.git_config.user_name, scope).with_context(
        || {
            format!(
                "Failed to set user.name for profile '{}' ({})",
                name, scope_str
            )
        },
    )?;
    println!(
        "  Set user.name to: {}",
        profile_to_apply.git_config.user_name.green()
    );

    set_git_config("user.email", &profile_to_apply.git_config.user_email, scope).with_context(
        || {
            format!(
                "Failed to set user.email for profile '{}' ({})",
                name, scope_str
            )
        },
    )?;
    println!(
        "  Set user.email to: {}",
        profile_to_apply.git_config.user_email.green()
    );

    if let Some(signing_key) = &profile_to_apply.git_config.user_signingkey {
        set_git_config("user.signingkey", signing_key, scope).with_context(|| {
            format!(
                "Failed to set user.signingkey for profile '{}' ({})",
                name, scope_str
            )
        })?;
        println!("  Set user.signingkey to: {}", signing_key.green());
    } else {
        // If the profile doesn't have a signing key, unset any existing one at this scope
        unset_git_config("user.signingkey", scope)
            .with_context(|| format!("Failed to unset user.signingkey ({})", scope_str))?;
        println!("  Unset user.signingkey (profile has no signing key specified).");
    }

    // TODO: Add logic for ssh_key and gpg_key if they influence git config directly (e.g. core.sshCommand, gpg.program)
    // For now, they are informational or for other tools.

    // Update current profile in gitp config
    config.current_profile = Some(name.clone());
    config
        .save()
        .context("Failed to save gitp configuration.")?;

    println!(
        "Successfully set '{}' as the active Git profile for {} scope.",
        name.green(),
        scope_str
    );
    println!(
        "gitp internal current profile also updated to '{}'.",
        name.green()
    );

    Ok(())
}

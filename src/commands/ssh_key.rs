use anyhow::{bail, Context, Result};
use colored::Colorize;

use crate::cli::SshKeyCommands;
use crate::config::Config;

pub fn execute(command: SshKeyCommands) -> Result<()> {
    match command {
        SshKeyCommands::Set { profile_name, key_path } => {
            set_ssh_key(profile_name, key_path)
        }
        SshKeyCommands::Remove { profile_name } => {
            remove_ssh_key(profile_name)
        }
        SshKeyCommands::Show { profile_name } => {
            show_ssh_key(profile_name)
        }
    }
}

fn set_ssh_key(profile_name: String, key_path: String) -> Result<()> {
    let mut config = Config::load().context("Failed to load configuration.")?;

    if !config.profiles.contains_key(&profile_name) {
        bail!("Profile '{}' not found.", profile_name.yellow());
    }

    // Validate key_path looks like a path (basic check)
    // More thorough validation (e.g. file existence) could be added here or in Profile::validate
    let path = std::path::PathBuf::from(key_path.clone());
    if !path.exists() {
        // Optionally, prompt if user wants to add a non-existent path, or just error
        bail!("SSH key path '{}' does not exist.", key_path.red());
    }
    // It's good practice to check if it's an absolute path or resolve it.
    // For simplicity, we'll store it as given, but real-world might need canonicalization.

    let profile = config.profiles.get_mut(&profile_name).unwrap(); // Should exist due to check above
    profile.ssh_key = Some(path);

    config.save().context("Failed to save configuration.")?;
    println!(
        "SSH key path for profile '{}' set to '{}'.",
        profile_name.cyan(),
        key_path.green()
    );
    Ok(())
}

fn remove_ssh_key(profile_name: String) -> Result<()> {
    let mut config = Config::load().context("Failed to load configuration.")?;

    if !config.profiles.contains_key(&profile_name) {
        bail!("Profile '{}' not found.", profile_name.yellow());
    }

    let profile = config.profiles.get_mut(&profile_name).unwrap();
    if profile.ssh_key.is_none() {
        println!(
            "Profile '{}' does not have an SSH key associated.",
            profile_name.cyan()
        );
        return Ok(());
    }

    profile.ssh_key = None;
    config.save().context("Failed to save configuration.")?;
    println!(
        "SSH key association removed from profile '{}'.",
        profile_name.cyan()
    );
    Ok(())
}

fn show_ssh_key(profile_name: String) -> Result<()> {
    let config = Config::load().context("Failed to load configuration.")?;

    match config.profiles.get(&profile_name) {
        Some(profile) => {
            if let Some(ssh_key_path) = &profile.ssh_key {
                println!(
                    "SSH key for profile '{}': {}",
                    profile_name.cyan(),
                    ssh_key_path.display().to_string().green()
                );
            } else {
                println!(
                    "Profile '{}' does not have an SSH key associated.",
                    profile_name.cyan()
                );
            }
        }
        None => {
            bail!("Profile '{}' not found.", profile_name.yellow());
        }
    }
    Ok(())
}

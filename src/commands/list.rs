use anyhow::Result;
use colored::Colorize;

use crate::config::{Config, Profile};

/// Execute the list command to show all profiles
pub fn execute(verbose: bool) -> Result<()> {
    let config = Config::load()?;

    if config.profiles.is_empty() {
        println!("No profiles found. Create one with 'gitp new <name>'");
        return Ok(());
    }

    let current_profile = config.current_profile.as_deref();

    if verbose {
        // Detailed view
        for (name, profile) in &config.profiles {
            print_profile_detailed(name, profile, current_profile);
            println!(); // Empty line between profiles
        }
    } else {
        // Simple list view
        println!("Available profiles:");
        println!();

        for name in config.profiles.keys() {
            if Some(name.as_str()) == current_profile {
                println!("  {} {}", "*".green().bold(), name.green().bold());
            } else {
                println!("    {}", name);
            }
        }

        println!();
        println!("{}", ("* = current profile" as &str).dimmed());
    }

    Ok(())
}

fn print_profile_detailed(name: &str, profile: &Profile, current_profile: Option<&str>) {
    // Header
    if Some(name) == current_profile {
        println!(
            "{} {} {}",
            "●".green().bold(),
            name.green().bold(),
            ("(current)" as &str).dimmed()
        );
    } else {
        println!("{} {}", "●".white(), name.bold());
    }

    // Git config
    println!("  {} {}", "Name:".cyan(), profile.git_config.user_name);
    println!("  {} {}", "Email:".cyan(), profile.git_config.user_email);

    // Optional fields
    if let Some(ref signing_key) = profile.git_config.user_signingkey {
        println!("  {} {}", "Signing Key:".cyan(), signing_key);
    }

    if let Some(ref ssh_key) = profile.ssh_key {
        println!("  {} {}", "SSH Key:".cyan(), ssh_key.display());
    }

    if let Some(ref gpg_key) = profile.gpg_key {
        println!("  {} {}", "GPG Key:".cyan(), gpg_key);
    }

    if !profile.custom_config.is_empty() {
        println!("  {}:", "Custom Config:".cyan());
        for (key, value) in &profile.custom_config {
            println!("    {} = {}", key, value);
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*; // Not strictly needed if only testing specific items explicitly
    use crate::config::Profile;
    // use std::collections::HashMap; // Not used in these specific tests yet

    #[test]
    fn test_empty_profile_list() {
        // This would need proper mocking of Config::load()
        // For now, this is a placeholder to show the testing approach
        // Example of how it might look with a mock:
        // let mut mock_config = Config::default();
        // let result = execute_with_config(&mock_config, false); // execute would need refactoring
        // assert!(result.is_ok());
        // Check stdout for "No profiles found"
    }

    #[test]
    fn test_profile_formatting() {
        let _profile = Profile::new(
            // Mark as unused for now if not asserted against
            "test".to_string(),
            "Test User".to_string(),
            "test@example.com".to_string(),
        );

        // Test that profile details are formatted correctly
        // This would be expanded with actual formatting tests. For example:
        // let mut output = Vec::new(); // To capture output
        // print_profile_detailed_to_writer("test", &profile, None, &mut output).unwrap();
        // let output_str = String::from_utf8(output).unwrap();
        // assert!(output_str.contains("Test User"));
    }
}

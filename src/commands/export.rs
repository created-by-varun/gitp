use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::io::{self, Write};

use crate::config::Config;

pub fn execute(profile_name: String, output_path: Option<String>) -> Result<()> {
    let config = Config::load().context("Failed to load configuration.")?;

    let profile = config
        .profiles
        .get(&profile_name)
        .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found.", profile_name.yellow()))?;

    let toml_string =
        toml::to_string_pretty(profile).context("Failed to serialize profile to TOML.")?;

    match output_path {
        Some(path) => {
            fs::write(&path, toml_string)
                .with_context(|| format!("Failed to write profile to file '{}'", path))?;
            println!(
                "Profile '{}' exported successfully to '{}'.",
                profile_name.cyan(),
                path.green()
            );
        }
        None => {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            handle
                .write_all(toml_string.as_bytes())
                .context("Failed to write profile to stdout.")?;
            // Add a newline if stdout is a tty, for better terminal output
            if atty::is(atty::Stream::Stdout) {
                handle
                    .write_all(b"\n")
                    .context("Failed to write newline to stdout.")?;
            }
        }
    }

    Ok(())
}

// SSH Config Management Logic

use anyhow::{Context, Result};
use std::fs::{OpenOptions};
use std::io::{Write};
use std::path::{Path, PathBuf};

pub(crate) const SSH_CONFIG_HEADER_START: &str = "# BEGIN MANAGED BY GITP";
pub(crate) const SSH_CONFIG_HEADER_END: &str = "# END MANAGED BY GITP";

/// Returns the default path to the user's SSH config file.
pub(crate) fn get_ssh_config_path() -> Result<PathBuf> {
    let home_dir = dirs::home_dir().context("Failed to get home directory.")?;
    Ok(home_dir.join(".ssh").join("config"))
}

/// Reads the content of the SSH config file.
/// If the file does not exist, returns an empty string.
pub(crate) fn read_ssh_config(config_path: &Path) -> Result<String> {
    if !config_path.exists() {
        return Ok(String::new());
    }
    fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read SSH config file from {:?}", config_path))
}

/// Generates a standard SSH config entry string for a given host and identity file.
pub(crate) fn generate_ssh_config_entry(
    host: &str,
    identity_file_path: &Path,
    user: Option<&str>,
) -> String {
    let user = user.unwrap_or("git");
    // Ensure the path is absolute and correctly formatted for the SSH config
    // SSH config typically expects absolute paths, especially if `~` is not expanded by SSH itself in all contexts.
    // However, `IdentityFile` does expand `~`, so we can use it if the path starts with `~`.
    // For simplicity and robustness, we'll try to provide an absolute path if not already.
    let identity_file_str = identity_file_path.to_string_lossy();

    format!(
        "Host {host}\n    HostName {host}\n    User {user}\n    IdentityFile {identity_file_str}\n    IdentitiesOnly yes\n",
        host = host,
        user = user,
        identity_file_str = identity_file_str
    )
}

use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Updates the SSH config file with entries managed by gitp.
/// It ensures that only entries from currently defined gitp profiles with SSH are present
/// within a specially marked block in the SSH config file.
pub fn update_ssh_config(managed_entries: &[(String, PathBuf, Option<String>)]) -> Result<()> {
    let config_path = get_ssh_config_path()?;
    let ssh_dir = config_path.parent().ok_or_else(|| anyhow::anyhow!("Invalid SSH config path, cannot get parent directory."))?;

    // Ensure .ssh directory exists with correct permissions (0700)
    if !ssh_dir.exists() {
        fs::create_dir_all(ssh_dir).with_context(|| format!("Failed to create .ssh directory at {:?}", ssh_dir))?;
        #[cfg(unix)]
        fs::set_permissions(ssh_dir, fs::Permissions::from_mode(0o700))
            .with_context(|| format!("Failed to set permissions for .ssh directory at {:?}", ssh_dir))?;
    }

    let original_config_content = read_ssh_config(&config_path)?;
    let mut new_config_content = original_config_content.clone();

    let mut new_gitp_block_content = String::new();
    if !managed_entries.is_empty() {
        new_gitp_block_content.push_str(SSH_CONFIG_HEADER_START);
        new_gitp_block_content.push('\n');
        for (host, key_path, user) in managed_entries {
            new_gitp_block_content.push_str(&generate_ssh_config_entry(host, key_path, user.as_deref()));
        }
        new_gitp_block_content.push_str(SSH_CONFIG_HEADER_END);
        new_gitp_block_content.push('\n');
    }

    let start_marker_idx = original_config_content.find(SSH_CONFIG_HEADER_START);
    let end_marker_idx = original_config_content.rfind(SSH_CONFIG_HEADER_END);

    match (start_marker_idx, end_marker_idx) {
        (Some(start_idx), Some(end_idx)) if start_idx < end_idx => {
            // Block found, replace it
            let end_of_block = end_idx + SSH_CONFIG_HEADER_END.len();
            // Include newline after block if it exists
            let end_of_block_with_newline = original_config_content.get(end_of_block..)
                .and_then(|s| s.chars().next().filter(|&c| c == '\n'))
                .map_or(end_of_block, |_| end_of_block + 1);
            
            new_config_content.replace_range(start_idx..end_of_block_with_newline, &new_gitp_block_content);
        }
        _ => {
            // Block not found or malformed, append if there's new content
            if !new_gitp_block_content.is_empty() {
                if !new_config_content.is_empty() && !new_config_content.ends_with('\n') {
                    new_config_content.push('\n'); // Ensure a newline before appending new block
                }
                new_config_content.push_str(&new_gitp_block_content);
            }
        }
    }
    
    // Trim multiple blank lines and ensure a single trailing newline
    let mut temp_lines: Vec<String> = Vec::new();
    let mut last_line_was_empty = false;
    for line_str in new_config_content.lines() {
        if line_str.trim().is_empty() {
            if !last_line_was_empty {
                temp_lines.push(String::new()); // Add a single representation of an empty line
            }
            last_line_was_empty = true;
        } else {
            temp_lines.push(line_str.to_string()); // Store owned string
            last_line_was_empty = false;
        }
    }

    let mut result_string = temp_lines.join("\n");

    if !result_string.is_empty() {
        // Remove all existing trailing newlines to normalize
        while result_string.ends_with('\n') {
            result_string.pop();
        }
        // Add exactly one trailing newline
        result_string.push('\n');
    }
    // If, after processing, result_string is empty (e.g., original was all whitespace or empty),
    // it will remain empty, which is correct.

    new_config_content = result_string;


    // Write the new config if it has changed
    if new_config_content.trim() != original_config_content.trim() || (!config_path.exists() && !new_config_content.is_empty()) {
        // Backup existing config file
        if config_path.exists() {
            let backup_path = config_path.with_extension("bak");
            fs::copy(&config_path, &backup_path).with_context(|| {
                format!("Failed to backup SSH config file to {:?}", backup_path)
            })?;
        }

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&config_path)
            .with_context(|| format!("Failed to open SSH config file for writing at {:?}", config_path))?;
        file.write_all(new_config_content.as_bytes())
            .with_context(|| format!("Failed to write to SSH config file at {:?}", config_path))?;

        #[cfg(unix)]
        fs::set_permissions(&config_path, fs::Permissions::from_mode(0o600))
            .with_context(|| format!("Failed to set permissions for SSH config file at {:?}", config_path))?;
        
        println!("SSH config updated at {:?}", config_path);
    } else {
        // println!("SSH config at {:?} is already up to date.", config_path);
    }

    Ok(())
}

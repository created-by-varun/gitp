use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GitConfigScope {
    Local,
    Global,
}

impl GitConfigScope {
    fn as_arg(&self) -> &'static str {
        match self {
            GitConfigScope::Local => "--local",
            GitConfigScope::Global => "--global",
        }
    }
}

/// Runs a git command with the given arguments.
fn run_git_command(args: &[&str]) -> Result<()> {
    let command_str = format!("git {}", args.join(" "));
    // println!("Executing: {}", command_str.dimmed()); // Optional: for debugging

    let output = Command::new("git")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .with_context(|| format!("Failed to execute command: {}", command_str))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "Git command failed: {}\\n{}",
            command_str.red(),
            stderr.trim().red()
        );
    }
    Ok(())
}

/// Sets a Git configuration value.
pub fn set_git_config(key: &str, value: &str, scope: GitConfigScope) -> Result<()> {
    run_git_command(&["config", scope.as_arg(), key, value]).with_context(|| {
        format!(
            "Failed to set Git config {} to '{}' ({:?})",
            key, value, scope
        )
    })
}

/// Unsets a Git configuration value.
/// It's not an error if the key doesn't exist.
pub fn unset_git_config(key: &str, scope: GitConfigScope) -> Result<()> {
    let args = &["config", scope.as_arg(), "--unset", key];
    let command_str = format!("git {}", args.join(" "));

    let output = Command::new("git")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .with_context(|| {
            format!(
                "Failed to execute command to unset Git config: {}",
                command_str
            )
        })?;

    if output.status.success() {
        // Key was found and successfully removed
        Ok(())
    } else if output.status.code() == Some(5) {
        // Key was not found, which is fine for an unset operation.
        Ok(())
    } else {
        // Another error occurred
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "Failed to unset Git config key '{}' ({:?}): {}\n{}",
            key,
            scope,
            command_str.red(),
            stderr.trim().red()
        );
    }
}

/// Gets a Git configuration value.
/// Returns Ok(None) if the key is not set.
pub fn get_git_config(key: &str, scope: GitConfigScope) -> Result<Option<String>> {
    let args = &["config", scope.as_arg(), "--get", key];
    let command_str = format!("git {}", args.join(" "));

    let output = Command::new("git")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .with_context(|| {
            format!(
                "Failed to execute command to get Git config: {}",
                command_str
            )
        })?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if stdout.is_empty() {
            Ok(None) // Key exists but has no value (shouldn't happen for user.name/email)
        } else {
            Ok(Some(stdout))
        }
    } else {
        // Exit code 1 usually means the key was not found, which is not an error for us.
        // Other exit codes might indicate a real problem.
        let stderr = String::from_utf8_lossy(&output.stderr);
        if output.status.code() == Some(1) && stderr.is_empty() {
            // Key not found
            Ok(None)
        } else {
            bail!(
                "Failed to get Git config for key '{}' ({:?}): {}\\n{}",
                key,
                scope,
                command_str.red(),
                stderr.trim().red()
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests will interact with the actual Git configuration.
    // They are useful for development but might need to be run carefully
    // or adapted for CI environments (e.g., by running in a temporary Git repo).

    const TEST_KEY_LOCAL: &str = "gitp.test.localsetting";
    const TEST_KEY_GLOBAL: &str = "gitp.test.globalsetting";
    const TEST_VALUE: &str = "testvalue123";

    fn cleanup_git_config(key: &str, scope: GitConfigScope) {
        let _ = unset_git_config(key, scope); // Ignore result, just cleanup
    }

    #[test]
    fn test_set_get_unset_local_config() -> Result<()> {
        cleanup_git_config(TEST_KEY_LOCAL, GitConfigScope::Local);

        // Set
        set_git_config(TEST_KEY_LOCAL, TEST_VALUE, GitConfigScope::Local)?;

        // Get
        let val = get_git_config(TEST_KEY_LOCAL, GitConfigScope::Local)?;
        assert_eq!(val, Some(TEST_VALUE.to_string()));

        // Unset
        unset_git_config(TEST_KEY_LOCAL, GitConfigScope::Local)?;

        // Get again
        let val_after_unset = get_git_config(TEST_KEY_LOCAL, GitConfigScope::Local)?;
        assert_eq!(val_after_unset, None);

        cleanup_git_config(TEST_KEY_LOCAL, GitConfigScope::Local);
        Ok(())
    }

    #[test]
    #[serial_test::serial]
    fn test_set_get_unset_global_config() -> Result<()> {
        // These tests modify global Git config, use with caution or mock if possible.
        // For now, ensure cleanup.
        cleanup_git_config(TEST_KEY_GLOBAL, GitConfigScope::Global);

        // Set
        set_git_config(TEST_KEY_GLOBAL, TEST_VALUE, GitConfigScope::Global)?;

        // Get
        let val = get_git_config(TEST_KEY_GLOBAL, GitConfigScope::Global)?;
        assert_eq!(val, Some(TEST_VALUE.to_string()));

        // Unset
        unset_git_config(TEST_KEY_GLOBAL, GitConfigScope::Global)?;

        // Get again
        let val_after_unset = get_git_config(TEST_KEY_GLOBAL, GitConfigScope::Global)?;
        assert_eq!(val_after_unset, None);

        cleanup_git_config(TEST_KEY_GLOBAL, GitConfigScope::Global);
        Ok(())
    }

    #[test]
    fn test_get_non_existent_config() -> Result<()> {
        let non_existent_key = "gitp.test.nonexistentkey";
        cleanup_git_config(non_existent_key, GitConfigScope::Local);
        cleanup_git_config(non_existent_key, GitConfigScope::Global);

        let val_local = get_git_config(non_existent_key, GitConfigScope::Local)?;
        assert_eq!(val_local, None);

        let val_global = get_git_config(non_existent_key, GitConfigScope::Global)?;
        assert_eq!(val_global, None);
        Ok(())
    }
}

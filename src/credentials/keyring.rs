// src/credentials/keyring.rs

use anyhow::{Context, Result};
use keyring::Entry;

const KEYRING_SERVICE_PREFIX: &str = "gitp_https_token_for_";

/// Stores an HTTPS token in the system keychain.
/// `target_host` is used to construct the service name (e.g., "github.com").
/// `username_or_profile` is used as the account name for the entry.
pub fn store_token(target_host: &str, username_or_profile: &str, token: &str) -> Result<()> {
    let service_name = format!("{}{}", KEYRING_SERVICE_PREFIX, target_host);
    let entry = Entry::new(&service_name, username_or_profile)?;
    entry.set_password(token).with_context(|| {
        format!(
            "Failed to store token for host '{}', user/profile '{}' in keychain",
            target_host, username_or_profile
        )
    })
}

/// Retrieves an HTTPS token from the system keychain.
/// `target_host` is used to construct the service name.
/// `username_or_profile` is the account name for the entry.
#[allow(dead_code)]
pub fn retrieve_token(target_host: &str, username_or_profile: &str) -> Result<String> {
    let service_name = format!("{}{}", KEYRING_SERVICE_PREFIX, target_host);
    let entry = Entry::new(&service_name, username_or_profile)?;
    entry.get_password().with_context(|| {
        format!(
            "Failed to retrieve token for host '{}', user/profile '{}' from keychain",
            target_host, username_or_profile
        )
    })
}

/// Deletes an HTTPS token from the system keychain.
/// `target_host` is used to construct the service name.
/// `username_or_profile` is the account name for the entry.
pub fn delete_token(target_host: &str, username_or_profile: &str) -> Result<()> {
    let service_name = format!("{}{}", KEYRING_SERVICE_PREFIX, target_host);
    let entry = Entry::new(&service_name, username_or_profile)?;
    entry.delete_password().with_context(|| {
        format!(
            "Failed to delete token for host '{}', user/profile '{}' from keychain",
            target_host, username_or_profile
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    // Use a unique service and account for tests to avoid conflicts
    const TEST_HOST: &str = "example_test_host.com";
    const TEST_USER: &str = "test_user_for_keyring_module";
    const TEST_TOKEN: &str = "test_token_data_12345!@#$%";

    // Helper to clean up entry if it exists
    fn cleanup_test_entry() {
        let _ = delete_token(TEST_HOST, TEST_USER); // Ignore error if not found
    }

    #[test]
    fn test_store_retrieve_delete_token() -> Result<()> {
        cleanup_test_entry(); // Ensure clean state

        // Test store
        store_token(TEST_HOST, TEST_USER, TEST_TOKEN).context("Test: Failed to store token")?;

        // Test retrieve
        let retrieved_token =
            retrieve_token(TEST_HOST, TEST_USER).context("Test: Failed to retrieve token")?;
        assert_eq!(
            retrieved_token, TEST_TOKEN,
            "Retrieved token does not match stored token"
        );

        // Test delete
        delete_token(TEST_HOST, TEST_USER).context("Test: Failed to delete token")?;

        // Verify deletion by trying to retrieve again (should fail)
        match retrieve_token(TEST_HOST, TEST_USER) {
            Ok(_) => panic!("Token was retrieved after it should have been deleted."),
            Err(e) => {
                eprintln!("Debug: Error after delete: {:?}", e);
                if let Some(source) = e.source() {
                    eprintln!("Debug: Error source: {:?}", source);
                }
                // We expect an error. The exact error message might vary by backend,
                // so we check if it contains typical phrases for "not found".
                let err_msg = e.to_string().to_lowercase();
                let source_msg = e
                    .source()
                    .map(|s| s.to_string().to_lowercase())
                    .unwrap_or_default();

                assert!(
                    err_msg.contains("not found") || 
                    err_msg.contains("no such item") || 
                    err_msg.contains("could not be found") ||
                    err_msg.contains("element not found") || 
                    source_msg.contains("not found") ||     // Check source too
                    source_msg.contains("no such item") ||
                    source_msg.contains("element not found") ||
                    source_msg.contains("no matching entry") || // From keyring::Error::NoEntry display
                    source_msg.contains("errsecitemnotfound") || // macOS specific error code for item not found
                    err_msg.contains("item_not_found"), // keyring crate specific error variant
                    "Error message did not indicate token not found. Full error: '{}', Source: '{}'", err_msg, source_msg
                );
            }
        }
        Ok(())
    }
}

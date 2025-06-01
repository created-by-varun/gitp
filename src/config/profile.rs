use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    /// Profile name (identifier)
    pub name: String,

    /// Git configuration
    pub git_config: GitConfig,

    /// Associated SSH key path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_key: Option<PathBuf>,

    /// GPG signing key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpg_key: Option<String>,

    /// HTTPS credentials (future implementation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub https_credentials: Option<HttpsCredentials>,

    /// Custom git configuration options
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom_config: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitConfig {
    /// Git user.name
    #[serde(rename = "name")]
    pub user_name: String,

    /// Git user.email
    #[serde(rename = "email")]
    pub user_email: String,

    /// Git user.signingkey
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_signingkey: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpsCredentials {
    /// Host (e.g., github.com)
    pub host: String,

    /// Username
    pub username: String,

    /// Credential type
    pub credential_type: CredentialType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum CredentialType {
    /// Personal access token (stored in config - not recommended)
    Token(String),

    /// Reference to system keychain
    KeychainRef(String),
}

impl Profile {
    /// Create a new profile with minimal configuration
    pub fn new(name: String, user_name: String, user_email: String) -> Self {
        Self {
            name,
            git_config: GitConfig {
                user_name,
                user_email,
                user_signingkey: None,
            },
            ssh_key: None,
            gpg_key: None,
            https_credentials: None,
            custom_config: HashMap::new(),
        }
    }

    /// Validate profile configuration
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.name.is_empty() {
            return Err(ValidationError::EmptyName);
        }

        if self.git_config.user_name.is_empty() {
            return Err(ValidationError::EmptyUserName);
        }

        if self.git_config.user_email.is_empty() {
            return Err(ValidationError::EmptyEmail);
        }

        // Enhanced email validation using regex
        // This regex is a common pattern, not strictly RFC 5322 compliant but good for most cases.
        let email_regex_str = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";
        let email_regex = Regex::new(email_regex_str).unwrap(); // In a real app, handle unwrap better or use once_cell

        if !email_regex.is_match(&self.git_config.user_email) {
            return Err(ValidationError::InvalidEmail(
                self.git_config.user_email.clone(),
            ));
        }

        // Validate SSH key path if provided
        if let Some(ref ssh_key) = self.ssh_key {
            if !ssh_key.exists() {
                return Err(ValidationError::SshKeyNotFound(ssh_key.clone()));
            }
        }

        // Validate GPG key format if provided
        if let Some(ref gpg_key_id) = self.gpg_key {
            if gpg_key_id.is_empty() {
                // An explicitly provided GPG key ID should not be empty.
                // If no GPG key is intended, gpg_key should be None.
                return Err(ValidationError::InvalidGpgKeyFormat(gpg_key_id.clone()));
            }
            // Regex for 8, 16, or 40 hex characters
            let gpg_key_regex_str = r"^[0-9A-Fa-f]{8}([0-9A-Fa-f]{8})?([0-9A-Fa-f]{24})?$";
            let gpg_key_regex = Regex::new(gpg_key_regex_str).unwrap(); // Handle unwrap better in prod

            if !gpg_key_regex.is_match(gpg_key_id) {
                return Err(ValidationError::InvalidGpgKeyFormat(gpg_key_id.clone()));
            }
        }

        // Validate HTTPS credentials if provided
        if let Some(creds) = &self.https_credentials {
            if creds.host.trim().is_empty() {
                return Err(ValidationError::EmptyHttpsHost);
            }
            if creds.username.trim().is_empty() {
                return Err(ValidationError::EmptyHttpsUsername);
            }
            match &creds.credential_type {
                CredentialType::Token(token) => {
                    if token.trim().is_empty() {
                        return Err(ValidationError::EmptyHttpsToken);
                    }
                }
                CredentialType::KeychainRef(keychain_ref) => {
                    if keychain_ref.trim().is_empty() {
                        return Err(ValidationError::EmptyHttpsKeychainRef);
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Profile name cannot be empty")]
    EmptyName,

    #[error("User name cannot be empty")]
    EmptyUserName,

    #[error("Email cannot be empty")]
    EmptyEmail,

    #[error("Invalid email format: {0}")]
    InvalidEmail(String),

    #[error("SSH key not found: {0}")]
    SshKeyNotFound(PathBuf),

    #[error("Invalid GPG key format: {0}. Expected 8, 16, or 40 hex characters.")]
    InvalidGpgKeyFormat(String),

    #[error("HTTPS credentials host cannot be empty")]
    EmptyHttpsHost,

    #[error("HTTPS credentials username cannot be empty")]
    EmptyHttpsUsername,

    #[error("HTTPS credentials token cannot be empty when type is Token")]
    EmptyHttpsToken,

    #[error("HTTPS credentials keychain reference cannot be empty when type is KeychainRef")]
    EmptyHttpsKeychainRef,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_creation() {
        let profile = Profile::new(
            "work".to_string(),
            "John Doe".to_string(),
            "john@company.com".to_string(),
        );

        assert_eq!(profile.name, "work");
        assert_eq!(profile.git_config.user_name, "John Doe");
        assert_eq!(profile.git_config.user_email, "john@company.com");
        assert!(profile.ssh_key.is_none());
    }

    #[test]
    fn test_profile_validation() {
        let mut profile = Profile::new(
            "test".to_string(),
            "Test User".to_string(),
            "test@example.com".to_string(),
        );

        assert!(profile.validate().is_ok());

        profile.git_config.user_email = "invalid-email".to_string();
        assert!(matches!(
            profile.validate(),
            Err(ValidationError::InvalidEmail(_))
        ));
    }

    #[test]
    fn test_https_credentials_validation() {
        let base_profile = |host: &str, username: &str, cred_type: CredentialType| {
            let mut p = Profile::new(
                "test_https".to_string(),
                "Test User".to_string(),
                "test@example.com".to_string(),
            );
            p.https_credentials = Some(HttpsCredentials {
                host: host.to_string(),
                username: username.to_string(),
                credential_type: cred_type,
            });
            p
        };

        // Valid: Token
        let profile_valid_token = base_profile(
            "github.com",
            "user1",
            CredentialType::Token("valid_token".to_string()),
        );
        assert!(profile_valid_token.validate().is_ok());

        // Valid: KeychainRef
        let profile_valid_keychain = base_profile(
            "gitlab.com",
            "user2",
            CredentialType::KeychainRef("valid_ref".to_string()),
        );
        assert!(profile_valid_keychain.validate().is_ok());

        // Invalid: Empty Host
        let profile_empty_host =
            base_profile(" ", "user3", CredentialType::Token("token".to_string()));
        assert!(matches!(
            profile_empty_host.validate(),
            Err(ValidationError::EmptyHttpsHost)
        ));

        // Invalid: Empty Username
        let profile_empty_username = base_profile(
            "bitbucket.org",
            " ",
            CredentialType::Token("token".to_string()),
        );
        assert!(matches!(
            profile_empty_username.validate(),
            Err(ValidationError::EmptyHttpsUsername)
        ));

        // Invalid: Empty Token
        let profile_empty_token = base_profile(
            "dev.azure.com",
            "user4",
            CredentialType::Token(" ".to_string()),
        );
        assert!(matches!(
            profile_empty_token.validate(),
            Err(ValidationError::EmptyHttpsToken)
        ));

        // Invalid: Empty KeychainRef
        let profile_empty_keychain_ref = base_profile(
            "source.example.com",
            "user5",
            CredentialType::KeychainRef(" ".to_string()),
        );
        assert!(matches!(
            profile_empty_keychain_ref.validate(),
            Err(ValidationError::EmptyHttpsKeychainRef)
        ));

        // Valid: No HTTPS credentials
        let profile_no_https = Profile::new(
            "no_https".to_string(),
            "Test User".to_string(),
            "test@example.com".to_string(),
        );
        assert!(profile_no_https.validate().is_ok());
    }
}

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

    /// Create a profile builder
    pub fn builder(name: String) -> ProfileBuilder {
        ProfileBuilder::new(name)
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

        // Basic email validation
        if !self.git_config.user_email.contains('@') {
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
}

/// Builder for creating profiles
pub struct ProfileBuilder {
    name: String,
    user_name: Option<String>,
    user_email: Option<String>,
    user_signingkey: Option<String>,
    ssh_key: Option<PathBuf>,
    gpg_key: Option<String>,
    custom_config: HashMap<String, String>,
}

impl ProfileBuilder {
    pub fn new(name: String) -> Self {
        Self {
            name,
            user_name: None,
            user_email: None,
            user_signingkey: None,
            ssh_key: None,
            gpg_key: None,
            custom_config: HashMap::new(),
        }
    }

    pub fn user_name(mut self, name: String) -> Self {
        self.user_name = Some(name);
        self
    }

    pub fn user_email(mut self, email: String) -> Self {
        self.user_email = Some(email);
        self
    }

    pub fn user_signingkey(mut self, key: String) -> Self {
        self.user_signingkey = Some(key);
        self
    }

    pub fn ssh_key(mut self, path: PathBuf) -> Self {
        self.ssh_key = Some(path);
        self
    }

    pub fn gpg_key(mut self, key: String) -> Self {
        self.gpg_key = Some(key);
        self
    }

    pub fn custom_config(mut self, key: String, value: String) -> Self {
        self.custom_config.insert(key, value);
        self
    }

    pub fn build(self) -> Result<Profile, &'static str> {
        let user_name = self.user_name.ok_or("User name is required")?;
        let user_email = self.user_email.ok_or("User email is required")?;

        Ok(Profile {
            name: self.name,
            git_config: GitConfig {
                user_name,
                user_email,
                user_signingkey: self.user_signingkey,
            },
            ssh_key: self.ssh_key,
            gpg_key: self.gpg_key,
            https_credentials: None,
            custom_config: self.custom_config,
        })
    }
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
    fn test_profile_builder() {
        let profile = Profile::builder("personal".to_string())
            .user_name("Jane Doe".to_string())
            .user_email("jane@example.com".to_string())
            .ssh_key(PathBuf::from("~/.ssh/id_rsa"))
            .build()
            .unwrap();

        assert_eq!(profile.name, "personal");
        assert_eq!(profile.git_config.user_name, "Jane Doe");
        assert_eq!(profile.ssh_key, Some(PathBuf::from("~/.ssh/id_rsa")));
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
}

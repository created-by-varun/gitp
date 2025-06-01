pub mod profile;
pub use profile::*;

use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Config {
    pub profiles: HashMap<String, Profile>,
    pub current_profile: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        // TODO: Implement actual loading from file
        Ok(Self {
            profiles: HashMap::new(),
            current_profile: None,
        })
    }
}

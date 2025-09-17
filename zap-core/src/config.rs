use std::{fmt, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Parsing(toml::de::Error),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "IO error: {}", e),
            ConfigError::Parsing(e) => write!(f, "TOML parse error: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<std::io::Error> for ConfigError {
    fn from(value: std::io::Error) -> Self {
        ConfigError::Io(value)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(value: toml::de::Error) -> Self {
        ConfigError::Parsing(value)
    }
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Config {
    pub site: Option<SiteConfig>,
    pub home: Option<HomeConfig>,
}

impl Config {
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let data = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&data)?;

        Ok(config)
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(default)]
pub struct SiteConfig {
    pub title: Option<String>,
    pub tagline: Option<String>,
    pub secondary_tagline: Option<String>,
    pub small_tag: Option<String>,
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            title: Some("Zap".into()),
            tagline: Some("A modern static site generator that creates beautiful project websites with minimal configuration".to_string()),
            secondary_tagline: None,
            small_tag: None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(default)]
pub struct HomeConfig {
    pub hero: bool,
    pub primary_action: Option<Link>,
    pub secondary_action: Option<Link>,
    #[serde(default)]
    pub features: Vec<Feature>,
}

impl Default for HomeConfig {
    fn default() -> Self {
        Self {
            hero: true,
            primary_action: None,
            secondary_action: None,
            features: Vec::new(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Link {
    pub text: String,
    pub link: String,
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Feature {
    pub title: String,
    pub description: String,
}

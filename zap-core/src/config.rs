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

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct SiteConfig {
    pub title: Option<String>,
    pub tagline: Option<String>,
    pub secondary_tagline: Option<String>,
    pub small_tag: Option<String>,
}

impl SiteConfig {
    pub fn with_defaults(mut self) -> Self {
        if self.title.is_none() {
            self.title = Some("Zap".to_string());
        }
        if self.tagline.is_none() {
            self.tagline = Some("A modern static site generator".to_string());
        }
        if self.small_tag.is_none() {
            self.small_tag = Some("Open Source • Zero Configuration".to_string());
        }
        self
    }
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct HomeConfig {
    pub primary_action: Option<Link>,
    pub secondary_action: Option<Link>,
    #[serde(default)]
    pub features: Vec<Feature>,
}

impl HomeConfig {
    pub fn with_defaults(mut self) -> Self {
        if self.primary_action.is_none() {
            self.primary_action = Some(Link {
                text: "Get Started".to_string(),
                link: "/getting-started".to_string(),
            });
        }
        if self.secondary_action.is_none() {
            self.secondary_action = Some(Link {
                text: "Documentation".to_string(),
                link: "/docs".to_string(),
            });
        }
        self
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

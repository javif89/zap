use anyhow::Result;
use clap::ArgMatches;
use config::{Config as ConfigBuilder, Environment, File};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Complete configuration that merges CLI args, env vars, config files, and defaults
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ZapConfig {
    /// Build configuration
    pub build: BuildConfig,
    /// Site configuration (from zap-core)
    #[serde(flatten)]
    pub site: zap_core::config::Config,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BuildConfig {
    /// Source directory containing markdown files
    pub source: String,
    /// Output directory for generated site
    pub output: String,
    /// Theme directory
    pub theme: String,
    /// Configuration file path
    pub config: String,
    /// Host for dev server
    pub host: String,
    /// Port for dev server
    pub port: u16,
    /// Open browser automatically
    pub open: bool,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            source: "./site".to_string(),
            output: "./out".to_string(),
            theme: "./theme".to_string(),
            config: "./zap.toml".to_string(),
            host: "127.0.0.1".to_string(),
            port: 3000,
            open: false,
        }
    }
}

impl Default for ZapConfig {
    fn default() -> Self {
        Self {
            build: BuildConfig::default(),
            site: zap_core::config::Config::default(),
        }
    }
}

impl ZapConfig {
    /// Load configuration with cascading precedence:
    /// 1. CLI arguments (highest priority)
    /// 2. Environment variables (ZAP_*)
    /// 3. Configuration file
    /// 4. Defaults (lowest priority)
    pub fn load(args: &ArgMatches) -> Result<Self> {
        let config_file = args.get_one::<String>("config")
            .unwrap_or(&"./zap.toml".to_string())
            .clone();

        let mut builder = ConfigBuilder::builder();

        // 1. Start with defaults
        let defaults = Self::default();
        builder = builder.add_source(config::Config::try_from(&defaults)?);

        // 2. Add configuration file if it exists
        if Path::new(&config_file).exists() {
            builder = builder.add_source(File::with_name(&config_file.replace(".toml", "")));
        }

        // 3. Add environment variables with ZAP_ prefix
        builder = builder.add_source(
            Environment::with_prefix("ZAP")
                .prefix_separator("_")
                .separator("__") // Use double underscore for nested keys
        );

        // 4. Override with CLI arguments (highest priority)
        let mut cli_overrides = std::collections::HashMap::new();

        if let Some(source) = args.get_one::<String>("source") {
            cli_overrides.insert("build.source".to_string(), source.clone());
        }
        if let Some(output) = args.get_one::<String>("output") {
            cli_overrides.insert("build.output".to_string(), output.clone());
        }
        if let Some(theme) = args.get_one::<String>("theme") {
            cli_overrides.insert("build.theme".to_string(), theme.clone());
        }
        if let Some(config) = args.get_one::<String>("config") {
            cli_overrides.insert("build.config".to_string(), config.clone());
        }
        // Only override with CLI args that are actually defined for this command
        if let Some(host) = args.try_get_one::<String>("host").unwrap_or(None) {
            cli_overrides.insert("build.host".to_string(), host.clone());
        }
        if let Some(port) = args.try_get_one::<String>("port").unwrap_or(None) {
            if let Ok(port_num) = port.parse::<u16>() {
                cli_overrides.insert("build.port".to_string(), port_num.to_string());
            }
        }
        if args.try_get_one::<bool>("open").unwrap_or(None).unwrap_or(&false) == &true {
            cli_overrides.insert("build.open".to_string(), "true".to_string());
        }

        if !cli_overrides.is_empty() {
            builder = builder.add_source(config::Config::try_from(&cli_overrides)?);
        }

        // Build and deserialize
        let config = builder.build()?;
        let zap_config: ZapConfig = config.try_deserialize()?;

        Ok(zap_config)
    }

    /// Get just the site configuration for passing to zap-core
    pub fn site_config(&self) -> &zap_core::config::Config {
        &self.site
    }

    /// Get the build configuration
    pub fn build_config(&self) -> &BuildConfig {
        &self.build
    }
}

/// Load configuration specifically for build commands
pub fn load_build_config(args: &ArgMatches) -> Result<ZapConfig> {
    ZapConfig::load(args)
}

/// Load configuration specifically for serve commands
pub fn load_serve_config(args: &ArgMatches) -> Result<ZapConfig> {
    ZapConfig::load(args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Arg, Command};

    #[test]
    fn test_default_config() {
        let config = ZapConfig::default();
        assert_eq!(config.build.source, "./site");
        assert_eq!(config.build.output, "./out");
        assert_eq!(config.build.theme, "./theme");
        assert_eq!(config.build.port, 3000);
    }

    #[test]
    fn test_cli_args_override() {
        let app = Command::new("test")
            .arg(Arg::new("source").long("source").value_name("DIR"))
            .arg(Arg::new("output").long("output").value_name("DIR"))
            .arg(Arg::new("config").long("config").value_name("FILE"));

        let matches = app.try_get_matches_from(vec![
            "test",
            "--source", "/custom/source",
            "--output", "/custom/output",
        ]).unwrap();

        let config = ZapConfig::load(&matches).unwrap();
        assert_eq!(config.build.source, "/custom/source");
        assert_eq!(config.build.output, "/custom/output");
        // Should still have defaults for non-overridden values
        assert_eq!(config.build.theme, "./theme");
    }
}
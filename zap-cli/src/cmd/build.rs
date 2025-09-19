use anyhow::Result;
use clap::{Arg, ArgMatches, Command};
use std::path::Path;
use zap_core::build_site;
use crate::config::load_build_config;

pub fn add_build_args(command: Command) -> Command {
    command
        .arg(
            Arg::new("source")
                .short('s')
                .long("source")
                .value_name("DIR")
                .help("Source directory containing markdown files")
                .default_value("./site")
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("DIR")
                .help("Output directory for generated site")
                .default_value("./out")
        )
        .arg(
            Arg::new("theme")
                .short('t')
                .long("theme")
                .value_name("DIR")
                .help("Theme directory")
                .default_value("./theme")
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration file")
                .default_value("./zap.toml")
        )
}

pub fn make_subcommand() -> Command {
    add_build_args(Command::new("build"))
        .about("Build static site from markdown files")
}

pub fn execute(args: &ArgMatches) -> Result<()> {
    // Load cascading configuration
    let zap_config = load_build_config(args)?;
    let build_config = zap_config.build_config();

    let source_dir = Path::new(&build_config.source);
    let output_dir = Path::new(&build_config.output);
    let theme_dir = Path::new(&build_config.theme);

    // Build site using shared function (dev_mode will be false for production)
    build_site(&zap_config.site, source_dir, output_dir, theme_dir)?;

    println!("Site built successfully in {}", output_dir.display());

    Ok(())
}


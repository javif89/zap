use anyhow::Result;
use clap::{Arg, ArgMatches, Command};
use std::path::Path;
use zap_core::{NavItem, PageType, SiteBuilder, SiteScanner, config::Config};

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
    let source_dir = args.get_one::<String>("source").unwrap();
    let output_dir = args.get_one::<String>("output").unwrap();
    let theme_dir = args.get_one::<String>("theme").unwrap();
    let config_file = args.get_one::<String>("config").unwrap();

    // Read configuration
    let config = Config::read(config_file).unwrap_or_default();

    // SCAN - Discover content
    let scanner = SiteScanner::new(source_dir);
    let (pages, collections) = scanner.scan()?;

    // PROCESS - Build navigation from discovered content
    let source_path = Path::new(source_dir);
    let mut navigation: Vec<NavItem> = pages
        .iter()
        .filter_map(|p| match p.page_type {
            PageType::Home => None,
            PageType::Changelog => None,
            _ => Some(NavItem {
                text: p.title.clone(),
                link: p.url(source_path),
            }),
        })
        .collect();

    let collection_links: Vec<NavItem> = collections
        .iter()
        .map(|c| NavItem {
            text: title_case(&c.name),
            link: format!("/{}", c.url()),
        })
        .collect();

    navigation.extend(collection_links);

    let home_config = config.home.unwrap_or_default();
    let mut site_config = config.site.unwrap_or_default();
    let home_page = pages.iter().find(|p| matches!(p.page_type, PageType::Home));

    // Build config by filling in as much as we can from README.md
    // before going to defaults
    if site_config.title.is_none() {
        site_config.title = home_page
            .and_then(|home| home.get_first_heading())
            .or_else(|| Some("Zap".to_string())); // Final fallback
    }

    // Could do similar for tagline from first paragraph
    site_config.tagline = home_page.and_then(|home| home.get_first_paragraph());

    // BUILD - Assemble the site
    let mut builder = SiteBuilder::new()
        .source_dir(source_dir)
        .output_dir(output_dir)
        .theme_dir(theme_dir)
        .site_config(site_config)
        .home_config(home_config)
        .navigation(navigation);

    // Add discovered content
    for page in pages {
        builder = builder.add_page(page);
    }
    for collection in collections {
        builder = builder.add_collection(collection);
    }

    // RENDER - Generate output
    let site = builder.build()?;
    site.render_all()?;

    println!("Site built successfully in {}", output_dir);

    Ok(())
}

pub fn title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
use zap_core::{NavItem, PageType, SiteBuilder, SiteScanner, config::Config};

trait FluentString {
    fn title(&self) -> String;
}

impl<T: AsRef<str>> FluentString for T {
    fn title(&self) -> String {
        self.as_ref()
            .split_whitespace()
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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read configuration
    let config = Config::read("./zap.toml").unwrap_or_default();

    // SCAN - Discover content
    let scanner = SiteScanner::new("./site");
    let (pages, collections) = scanner.scan()?;

    // PROCESS - Build navigation from discovered content
    let source_dir = std::path::Path::new("./site");
    let mut navigation: Vec<NavItem> = pages
        .iter()
        .filter_map(|p| match p.page_type {
            PageType::Home => None,
            PageType::Changelog => None,
            _ => Some(NavItem {
                text: p.title.clone(),
                link: p.url(source_dir),
            }),
        })
        .collect();

    let collection_links: Vec<NavItem> = collections
        .iter()
        .map(|c| NavItem {
            text: c.name.clone().title(),
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
        .source_dir("./site")
        .output_dir("./out")
        .theme_dir("./theme")
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
    let mut site = builder.build()?;
    site.render_all()?;

    Ok(())
}

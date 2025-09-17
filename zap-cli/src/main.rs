use zap_core::{
    NavItem, Page, PageElement, PageType, SiteBuilder, SiteScanner,
    config::{Config, SiteConfig},
    markdown::get_page_structured,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read configuration
    let config = Config::read("./zap.toml").unwrap_or_default();

    // SCAN - Discover content
    let scanner = SiteScanner::new("./site");
    let (pages, collections) = scanner.scan()?;

    // PROCESS - Build navigation from discovered content
    let source_dir = std::path::Path::new("./site");
    let mut navigation = Vec::new();
    for page in &pages {
        navigation.push(NavItem {
            text: page.title.clone(),
            link: page.url(source_dir),
        });
    }
    for collection in &collections {
        navigation.push(NavItem {
            text: collection.name.clone(),
            link: format!("/{}", collection.url()),
        });
    }

    let mut home_config = config.home.unwrap_or_default();
    let mut site_config = config.site.unwrap_or_default();
    let home = pages.iter().find(|p| matches!(p.page_type, PageType::Home));

    if home.is_some() {
        let h = home
            .unwrap()
            .elements()
            .into_iter()
            .find_map(|el| match el {
                PageElement::Heading { level: 1, text } => Some(text),
                _ => None,
            });

        if h.is_some() {
            site_config.title = Some(h.unwrap());
        }
    }

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

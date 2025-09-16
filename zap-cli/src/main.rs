use zap_core::{
    NavItem, Page, PageType, SiteBuilder, SiteScanner,
    config::{Config, SiteConfig},
    markdown::get_page_structured,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read configuration
    let config = Config::read("./zap.toml").unwrap_or_default();

    // SCAN - Discover content
    let scanner = SiteScanner::new("./site");
    let (pages, collections) = scanner.scan()?;

    println!("Discovered:");
    println!("Pages");
    for p in &pages {
        println!("- {}", p.title);
    }

    println!("Collections");
    for c in &collections {
        println!("- {}", c.name);
    }

    if let Some(home) = pages
        .into_iter()
        .find(|p| matches!(p.page_type, PageType::Home))
    {
        println!("Home page");

        for el in home.elements() {
            println!("{:?}", el);
        }
    }

    // // PROCESS - Build navigation from discovered content
    // let mut navigation = Vec::new();
    // for page in &pages {
    //     navigation.push(NavItem {
    //         text: page.title.clone(),
    //         link: format!("/{}", page.url()),
    //     });
    // }
    // for collection in &collections {
    //     navigation.push(NavItem {
    //         text: collection.name.clone(),
    //         link: format!("/{}", collection.url()),
    //     });
    // }

    // // BUILD - Assemble the site
    // let mut builder = SiteBuilder::new()
    //     .source_dir("./site")
    //     .output_dir("./out")
    //     .theme_dir("./theme")
    //     .site_config(config.site.unwrap_or_default())
    //     .home_config(config.home.unwrap_or_default())
    //     .navigation(navigation);

    // // Add discovered content
    // for page in pages {
    //     builder = builder.add_page(page);
    // }
    // for collection in collections {
    //     builder = builder.add_collection(collection);
    // }

    // // RENDER - Generate output
    // let mut site = builder.build()?;
    // site.render_all()?;

    Ok(())
}

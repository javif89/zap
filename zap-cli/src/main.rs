use std::path::PathBuf;

use serde::Serialize;
use zap_core::{TemplateRenderer, config::Config};

#[derive(Serialize)]
struct NavItem {
    text: String,
    link: String,
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = Config::read("./zap.toml").unwrap_or_default();
    println!("{cfg:#?}");
    
    let mut renderer = TemplateRenderer::new("theme/**/*.html")?;

    let mut zap = zap_core::Zap::new(PathBuf::from("./site"));
    zap.scan();

    let mut navigation: Vec<NavItem> = Vec::new();

    for p in zap.pages() {
        navigation.push(NavItem {
            text: p.title.to_owned(),
            link: format!("/{}", p.url()),
        });
    }

    for c in zap.collections() {
        navigation.push(NavItem {
            text: c.name.to_owned(),
            link: format!("/{}", c.url()),
        });
    }

    // Build context
    let site_config = cfg.site.unwrap_or_default();
    let home_config = cfg.home.unwrap_or_default();
    
    renderer.add_to_context("site", &site_config);
    renderer.add_to_context("home", &home_config);
    renderer.add_to_context("secondary_nav", &navigation);

    let _ = std::fs::create_dir_all("out");

    let out = PathBuf::from("./out");
    println!("Pages");
    for p in zap.pages() {
        println!("{}: {:?}", p.title, p.page_type);
        println!("{} -> {}", p.path.display(), p.out_path().display());
        
        renderer.add_to_context("page_content", &zap.render_page(p));
        match renderer.render_to_file(p.template_name(), &out.join(p.out_path())) {
            Ok(_) => println!("Rendered successfully"),
            Err(e) => eprintln!("Failed to render: {e:?}"),
        }
    }

    println!("Collections");
    for c in zap.collections() {
        println!("{}", c.name);
        let mut page_links: Vec<NavItem> = Vec::new();
        for p in &c.pages {
            page_links.push(NavItem {
                text: p.title.to_owned(),
                link: format!("/{}", p.url()),
            });
        }

        for p in &c.pages {
            println!("{}: {:?}", p.title, p.page_type);
            println!("{} -> {}", p.path.display(), p.out_path().display());
            
            renderer.add_to_context("page_content", &zap.render_page(p));
            renderer.add_to_context("collection_pages", &page_links);
            match renderer.render_to_file("doc.html", &out.join(p.out_path())) {
                Ok(_) => println!("Rendered successfully"),
                Err(e) => eprintln!("Render err: {}", e),
            }
        }
    }
    
    Ok(())
}

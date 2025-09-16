use std::path::PathBuf;

use serde::Serialize;
use tera::{Context, Tera};
use zap_core::PageType;

#[derive(Serialize)]
struct NavItem {
    text: String,
    link: String,
}
fn main() {
    let tera = match Tera::new("theme/**/*.html") {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Parsing error(s): {}", e);
            std::process::exit(1);
        }
    };

    let mut zap = zap_core::Zap::new(PathBuf::from("./site"));
    zap.scan();

    let mut context = Context::new();
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

    context.insert("title", "This should be title");
    context.insert("secondary_nav", &navigation);

    let _ = std::fs::create_dir_all("out");

    let out = PathBuf::from("./out");
    println!("Pages");
    for p in zap.pages() {
        println!("{}: {:?}", p.title, p.page_type);
        println!("{} -> {}", p.path.display(), p.out_path().display());
        context.insert("page_content", &zap.render_page(p));
        let template = get_page_template(p);
        if let Ok(s) = tera.render(&template, &context) {
            let _ = std::fs::create_dir_all(out.join(p.out_path().with_file_name("")));
            match std::fs::write(out.join(p.out_path()), s) {
                Ok(_) => println!("Rendered successfully"),
                Err(e) => eprintln!("Render err: {}", e),
            }
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
            context.insert("page_content", &zap.render_page(p));
            context.insert("collection_pages", &page_links);
            if let Ok(s) = tera.render("doc.html", &context) {
                let _ = std::fs::create_dir_all(out.join(p.out_path().with_file_name("")));
                match std::fs::write(out.join(p.out_path()), s) {
                    Ok(_) => println!("Rendered successfully"),
                    Err(e) => eprintln!("Render err: {}", e),
                }
            }
        }
    }
}

fn get_page_template(page: &zap_core::Page) -> String {
    match page.page_type {
        PageType::Home => "home.html".into(),
        PageType::Changelog => "changelog.html".into(),
        _ => "page.html".into(),
    }
}

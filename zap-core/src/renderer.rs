use crate::markdown::parse_page;
use crate::site::Page;
use std::path::PathBuf;

pub fn render_page(scan_path: &PathBuf, page: &Page) -> String {
    let page_path = scan_path.join(&page.path);

    match parse_page(page_path.to_string_lossy().to_string().as_str()) {
        Ok(out) => out,
        Err(_) => String::from("Sadly failed"),
    }
}
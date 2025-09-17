use crate::markdown::{parse_page, render_elements_to_html};
use crate::site::Page;
use std::path::PathBuf;

pub fn render_page(_scan_path: &PathBuf, page: &Page) -> String {
    // Use the new structured renderer
    let elements = page.elements();
    render_elements_to_html(&elements)
}

// Keep the old renderer as a fallback for now
pub fn render_page_legacy(_scan_path: &PathBuf, page: &Page) -> String {
    // page.path is now absolute, so we use it directly
    match parse_page(page.path.to_string_lossy().to_string().as_str()) {
        Ok(out) => out,
        Err(e) => {
            eprintln!("Failed to parse page {}: {}", page.path.display(), e);
            String::from("Sadly failed")
        }
    }
}
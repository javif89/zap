use crate::markdown::render_elements_to_html;
use crate::site::Page;

pub fn render_page(page: &Page) -> String {
    // Use the new structured renderer
    let elements = page.elements();
    render_elements_to_html(&elements)
}
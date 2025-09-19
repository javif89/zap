pub mod builder;
pub mod config;
pub mod markdown;
pub mod renderer;
pub mod scanner;
pub mod site;
pub mod template;

// Re-export main types
pub use builder::{BuildError, NavItem, RenderError, Site, SiteBuilder, build_site};
pub use markdown::{
    InlineElement, ListItem, PageElement, get_page_structured, parse_page, render_elements_to_html,
    render_inline_elements_text, slugify,
};
pub use renderer::{Renderer, RenderContext};
pub use scanner::{ScanError, SiteScanner};
pub use site::{Collection, Page, PageType, Zap};
pub use template::{TemplateError, TemplateRenderer};

pub mod config;
pub mod markdown;
pub mod site;
pub mod renderer;
pub mod template;
pub mod builder;
pub mod scanner;

// Re-export main types
pub use markdown::parse_page;
pub use site::{Page, PageType, Collection, Zap};
pub use template::{TemplateRenderer, TemplateError};
pub use builder::{SiteBuilder, Site, NavItem, BuildError, RenderError};
pub use scanner::{SiteScanner, ScanError};

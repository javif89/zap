pub mod config;
pub mod markdown;
pub mod site;
pub mod renderer;

// Re-export main types
pub use markdown::parse_page;
pub use site::{Page, PageType, Collection, Zap};

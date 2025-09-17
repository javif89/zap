use std::path::{Path, PathBuf};
use std::collections::HashMap;
use serde::Serialize;
use serde_json;

use crate::config::{SiteConfig, HomeConfig};
use crate::site::{Page, Collection};
use crate::template::TemplateRenderer;

#[derive(Debug)]
pub enum BuildError {
    MissingSourceDir,
    InvalidPath(PathBuf),
    TemplateError(crate::template::TemplateError),
    ScanError(std::io::Error),
    SerializationError(serde_json::Error),
}

impl From<crate::template::TemplateError> for BuildError {
    fn from(err: crate::template::TemplateError) -> Self {
        BuildError::TemplateError(err)
    }
}

impl From<std::io::Error> for BuildError {
    fn from(err: std::io::Error) -> Self {
        BuildError::ScanError(err)
    }
}

impl From<serde_json::Error> for BuildError {
    fn from(err: serde_json::Error) -> Self {
        BuildError::SerializationError(err)
    }
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::MissingSourceDir => write!(f, "Source directory not specified"),
            BuildError::InvalidPath(p) => write!(f, "Invalid path: {}", p.display()),
            BuildError::TemplateError(e) => write!(f, "Template error: {}", e),
            BuildError::ScanError(e) => write!(f, "Scan error: {}", e),
            BuildError::SerializationError(e) => write!(f, "Serialization error: {}", e),
        }
    }
}

impl std::error::Error for BuildError {}

#[derive(Debug, Serialize)]
pub struct NavItem {
    pub text: String,
    pub link: String,
}

pub struct SiteContext {
    pub site: SiteConfig,
    pub home: Option<HomeConfig>,
    pub navigation: Vec<NavItem>,
    pub custom: HashMap<String, serde_json::Value>,
}

impl Default for SiteContext {
    fn default() -> Self {
        Self {
            site: SiteConfig::default(),
            home: None,
            navigation: Vec::new(),
            custom: HashMap::new(),
        }
    }
}

pub struct SiteBuilder {
    source_dir: Option<PathBuf>,
    output_dir: PathBuf,
    theme_dir: PathBuf,
    syntax_theme: String,
    pages: Vec<Page>,
    collections: Vec<Collection>,
    context: SiteContext,
}

impl SiteBuilder {
    pub fn new() -> Self {
        Self {
            source_dir: None,
            output_dir: PathBuf::from("./out"),
            theme_dir: PathBuf::from("./theme"),
            syntax_theme: "base16-ocean.dark".to_string(),
            pages: Vec::new(),
            collections: Vec::new(),
            context: SiteContext::default(),
        }
    }

    // Required configuration
    pub fn source_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.source_dir = Some(path.as_ref().to_path_buf());
        self
    }

    // Optional paths
    pub fn output_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.output_dir = path.as_ref().to_path_buf();
        self
    }

    pub fn theme_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.theme_dir = path.as_ref().to_path_buf();
        self
    }

    // Context configuration
    pub fn site_config(mut self, config: SiteConfig) -> Self {
        self.context.site = config;
        self
    }

    pub fn home_config(mut self, config: HomeConfig) -> Self {
        self.context.home = Some(config);
        self
    }

    pub fn navigation(mut self, items: Vec<NavItem>) -> Self {
        self.context.navigation = items;
        self
    }

    // Custom context data
    pub fn add_custom<T: Serialize>(mut self, key: &str, value: T) -> Result<Self, BuildError> {
        let json_value = serde_json::to_value(value)?;
        self.context.custom.insert(key.to_string(), json_value);
        Ok(self)
    }

    // Content management
    pub fn add_page(mut self, page: Page) -> Self {
        self.pages.push(page);
        self
    }

    pub fn add_collection(mut self, collection: Collection) -> Self {
        self.collections.push(collection);
        self
    }

    // Bulk content addition
    pub fn add_pages(mut self, pages: Vec<Page>) -> Self {
        self.pages.extend(pages);
        self
    }

    pub fn add_collections(mut self, collections: Vec<Collection>) -> Self {
        self.collections.extend(collections);
        self
    }

    // Syntax highlighting configuration
    pub fn syntax_theme<S: Into<String>>(mut self, theme: S) -> Self {
        self.syntax_theme = theme.into();
        self
    }

    // Build the site
    pub fn build(self) -> Result<Site, BuildError> {
        let source_dir = self.source_dir.ok_or(BuildError::MissingSourceDir)?;
        
        // Build template renderer with context
        let theme_glob = format!("{}/**/*.html", self.theme_dir.display());
        let mut renderer = TemplateRenderer::new(&theme_glob)?;
        
        // Add structured context to templates
        renderer.add_to_context("site", &self.context.site);
        if let Some(home) = &self.context.home {
            renderer.add_to_context("home", home);
        }
        renderer.add_to_context("navigation", &self.context.navigation);
        renderer.add_to_context("secondary_nav", &self.context.navigation); // Backward compat
        
        // Add all custom context
        for (key, value) in &self.context.custom {
            renderer.get_context_mut().insert(key, value);
        }
        
        Ok(Site {
            pages: self.pages,
            collections: self.collections,
            renderer,
            output_dir: self.output_dir,
            source_dir,
        })
    }
}

#[derive(Debug)]
pub enum RenderError {
    TemplateError(crate::template::TemplateError),
    IoError(std::io::Error),
}

impl From<crate::template::TemplateError> for RenderError {
    fn from(err: crate::template::TemplateError) -> Self {
        RenderError::TemplateError(err)
    }
}

impl From<std::io::Error> for RenderError {
    fn from(err: std::io::Error) -> Self {
        RenderError::IoError(err)
    }
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderError::TemplateError(e) => write!(f, "Template error: {}", e),
            RenderError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for RenderError {}

pub struct Site {
    pages: Vec<Page>,
    collections: Vec<Collection>,
    renderer: TemplateRenderer,
    output_dir: PathBuf,
    source_dir: PathBuf,
}

impl Site {
    pub fn pages(&self) -> &[Page] {
        &self.pages
    }

    pub fn collections(&self) -> &[Collection] {
        &self.collections
    }

    fn render_page(&self, page: &Page) -> String {
        let elements = page.elements();
        crate::markdown::render_elements_to_html(&elements)
    }

    fn page_out_path(&self, page: &Page) -> PathBuf {
        // Convert absolute path to relative path for output
        let relative_path = page.path
            .strip_prefix(&self.source_dir)
            .unwrap_or(&page.path);

        match &page.page_type {
            crate::site::PageType::Home => PathBuf::from("index.html"),
            crate::site::PageType::Changelog => PathBuf::from("changelog/index.html"),
            crate::site::PageType::Index => relative_path
                .with_file_name("")
                .with_extension("")
                .join("index.html"),
            _ => relative_path.with_extension("").join("index.html"),
        }
    }

    fn page_url(&self, page: &Page) -> String {
        self.page_out_path(page)
            .with_file_name("")
            .to_string_lossy()
            .to_string()
    }

    pub fn render_all(&mut self) -> Result<(), RenderError> {
        // Ensure output directory exists
        std::fs::create_dir_all(&self.output_dir)?;
        
        // Render all pages
        for page in &self.pages {
            let out_path = self.page_out_path(page);
            
            let content = self.render_page(page);
            self.renderer.add_to_context("page_content", &content);
            
            let output_path = self.output_dir.join(out_path);
            if let Err(e) = self.renderer.render_to_file(page.template_name(), &output_path) {
                eprintln!("Failed to render {}: {:?}", page.title, e);
            }
        }
        
        // Render all collections
        for collection in &self.collections {
            // Build collection navigation
            let mut page_links: Vec<NavItem> = Vec::new();
            for page in &collection.pages {
                page_links.push(NavItem {
                    text: page.title.clone(),
                    link: format!("/{}", self.page_url(page)),
                });
            }
            
            for page in &collection.pages {
                let out_path = self.page_out_path(page);
                
                let content = self.render_page(page);
                self.renderer.add_to_context("page_content", &content);
                self.renderer.add_to_context("collection_pages", &page_links);
                
                let output_path = self.output_dir.join(out_path);
                if let Err(e) = self.renderer.render_to_file("doc.html", &output_path) {
                    eprintln!("Failed to render {}: {:?}", page.title, e);
                }
            }
        }
        
        Ok(())
    }
}
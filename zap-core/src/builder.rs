use serde::Serialize;
use serde_json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::config::{HomeConfig, SiteConfig};
use crate::renderer::{RenderContext, Renderer};
use crate::site::{Collection, Page};
use crate::template::TemplateError;
use crate::{PageElement, PageType};

#[derive(Debug)]
pub enum BuildError {
    MissingSourceDir,
    InvalidPath(PathBuf),
    TemplateError(crate::template::TemplateError),
    ScanError(std::io::Error),
    SerializationError(serde_json::Error),
}

impl From<TemplateError> for BuildError {
    fn from(err: TemplateError) -> Self {
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

#[derive(Default)]
pub struct SiteContext {
    pub site: SiteConfig,
    pub home: Option<HomeConfig>,
    pub navigation: Vec<NavItem>,
    pub custom: HashMap<String, serde_json::Value>,
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

impl Default for SiteBuilder {
    fn default() -> Self {
        Self::new()
    }
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

        // Create renderer with global context
        let mut renderer = Renderer::new(&self.theme_dir)?;

        // Set global context once
        renderer.set_global_context("site", &self.context.site);
        renderer.set_global_context("navigation", &self.context.navigation);
        renderer.set_global_context("secondary_nav", &self.context.navigation); // Backward compat

        // Check for changelog and add to global
        let has_changelog = self
            .pages
            .iter()
            .any(|p| matches!(p.page_type, PageType::Changelog));
        renderer.set_global_context("has_changelog", &has_changelog);

        // Add any custom global context
        for (key, value) in &self.context.custom {
            renderer.set_global_context(key, value);
        }

        Ok(Site {
            pages: self.pages,
            collections: self.collections,
            renderer,
            output_dir: self.output_dir,
            source_dir,
            home_config: self.context.home,
        })
    }
}

#[derive(Debug)]
pub enum RenderError {
    TemplateError(TemplateError),
    IoError(std::io::Error),
}

impl From<TemplateError> for RenderError {
    fn from(err: TemplateError) -> Self {
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
    renderer: Renderer,
    output_dir: PathBuf,
    source_dir: PathBuf,
    home_config: Option<HomeConfig>,
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
        let relative_path = page
            .path
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

    fn render_home(&self, page: &Page, home_config: &HomeConfig) -> Result<(), RenderError> {
        let mut context = RenderContext::new();

        // Get page elements and potentially filter them
        let mut elements = page.elements();

        // If hero is enabled, remove first h1 and first paragraph
        if home_config.hero {
            let mut found_h1 = false;
            let mut found_paragraph_after_h1 = false;

            elements.retain(|element| {
                match element {
                    PageElement::Heading { level: 1, .. } if !found_h1 => {
                        found_h1 = true;
                        false // Remove first h1
                    }
                    PageElement::Paragraph { .. } if found_h1 && !found_paragraph_after_h1 => {
                        found_paragraph_after_h1 = true;
                        false // Remove first paragraph after h1
                    }
                    _ => true, // Keep everything else
                }
            });
        }

        // Render the filtered content
        let content = crate::markdown::render_elements_to_html(&elements);
        context.add_to_context("page_content", &content);

        // Home-specific config
        context.add_to_context("home", home_config);

        let html = self.renderer.render(page.template_name(), &context)?;

        let out_path = self.page_out_path(page);
        let output_path = self.output_dir.join(out_path);
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(output_path, html)?;

        Ok(())
    }

    fn render_changelog(&self, page: &Page) -> Result<(), RenderError> {
        let mut context = RenderContext::new();

        // Only page-specific content
        let content = self.render_page(page);
        context.add_to_context("page_content", &content);

        let releases: Vec<NavItem> = page
            .elements()
            .iter()
            .filter_map(|el| match el {
                // We're preferring convention here. The only H1 should
                // be the page title.
                PageElement::Heading { level: 1, .. } => None,
                PageElement::Heading { level: 2, content } => {
                    let text = crate::markdown::render_inline_elements_text(content);
                    let slug = crate::markdown::slugify(&text);
                    Some(NavItem {
                        text,
                        link: format!("#{}", slug),
                    })
                }
                _ => None,
            })
            .collect();
        context.add_to_context("releases", &releases);

        let html = self.renderer.render(page.template_name(), &context)?;

        let output_path = self.output_dir.join("changelog/index.html");
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(output_path, html)?;

        Ok(())
    }

    fn render_regular_page(&self, page: &Page) -> Result<(), RenderError> {
        let mut context = RenderContext::new();

        let content = self.render_page(page);
        context.add_to_context("page_content", &content);

        let html = self.renderer.render(page.template_name(), &context)?;

        let out_path = self.page_out_path(page);
        let output_path = self.output_dir.join(out_path);
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(output_path, html)?;

        Ok(())
    }

    pub fn render_all(&self) -> Result<(), RenderError> {
        // TODO: Should probably be a bit more sophisticated than this
        // Delete output dir if it exists
        // let _ = std::fs::remove_dir_all(&self.output_dir);
        // Ensure output directory exists
        std::fs::create_dir_all(&self.output_dir)?;

        // Render all pages
        for page in &self.pages {
            match page.page_type {
                PageType::Home => {
                    if let Some(ref home_config) = self.home_config {
                        self.render_home(page, home_config)?;
                    } else {
                        self.render_regular_page(page)?;
                    }
                }
                PageType::Changelog => self.render_changelog(page)?,
                _ => self.render_regular_page(page)?,
            }
        }

        // Render all collections
        for collection in &self.collections {
            // Build collection navigation
            let page_links: Vec<NavItem> = collection
                .pages
                .iter()
                .map(|page| NavItem {
                    text: page.title.clone(),
                    link: format!("/{}", self.page_url(page)),
                })
                .collect();

            for page in &collection.pages {
                let mut context = RenderContext::new();

                // Only page-specific data
                let content = self.render_page(page);
                context.add_to_context("page_content", &content);
                context.add_to_context("collection_pages", &page_links);

                // Get page headings for side nav
                let headings: Vec<NavItem> = page
                    .elements()
                    .iter()
                    .filter_map(|el| match el {
                        // We're preferring convention here. The only H1 should
                        // be the page title.
                        PageElement::Heading { level: 1, .. } => None,
                        PageElement::Heading { content, .. } => {
                            let text = crate::markdown::render_inline_elements_text(content);
                            let slug = crate::markdown::slugify(&text);
                            Some(NavItem {
                                text,
                                link: format!("#{}", slug),
                            })
                        }
                        _ => None,
                    })
                    .collect();
                context.add_to_context("on_this_page", &headings);

                let html = self.renderer.render("doc.html", &context)?;

                let out_path = self.page_out_path(page);
                let output_path = self.output_dir.join(out_path);
                if let Some(parent) = output_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(output_path, html)?;
            }
        }

        Ok(())
    }
}

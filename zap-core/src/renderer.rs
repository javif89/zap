use serde::Serialize;
use std::path::Path;
use crate::template::{TemplateRenderer, TemplateError};

pub struct Renderer {
    templates: TemplateRenderer,
    global_context: RenderContext,  // Global context set once
}

impl Renderer {
    pub fn new(theme_dir: &Path) -> Result<Self, TemplateError> {
        let theme_glob = format!("{}/**/*.html", theme_dir.display());
        Ok(Self {
            templates: TemplateRenderer::new(&theme_glob)?,
            global_context: RenderContext::new(),
        })
    }
    
    // Set global context that applies to all renders
    pub fn set_global_context<T: Serialize>(&mut self, key: &str, value: &T) {
        self.global_context.add_to_context(key, value);
    }
    
    // Render template to string with merged global + page context
    pub fn render(&self, template: &str, page_context: &RenderContext) -> Result<String, TemplateError> {
        // Merge global and page contexts
        let mut merged = self.global_context.clone();
        merged.merge(page_context);
        
        self.templates.render_with_context(template, &merged.inner)
    }
}

#[derive(Clone)]
pub struct RenderContext {
    inner: tera::Context,
}

impl Default for RenderContext {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderContext {
    pub fn new() -> Self {
        Self {
            inner: tera::Context::new(),
        }
    }
    
    pub fn add_to_context<T: Serialize>(&mut self, key: &str, value: &T) {
        self.inner.insert(key, value);
    }
    
    // Merge another context into this one
    pub fn merge(&mut self, other: &RenderContext) {
        // This extends self with all values from other
        self.inner.extend(other.inner.clone());
    }
}
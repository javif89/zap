use std::path::Path;
use serde::Serialize;
use tera::{Context, Tera};

#[derive(Debug)]
pub enum TemplateError {
    TeraError(tera::Error),
    IoError(std::io::Error),
}

impl From<tera::Error> for TemplateError {
    fn from(err: tera::Error) -> Self {
        TemplateError::TeraError(err)
    }
}

impl From<std::io::Error> for TemplateError {
    fn from(err: std::io::Error) -> Self {
        TemplateError::IoError(err)
    }
}

impl std::fmt::Display for TemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateError::TeraError(e) => write!(f, "Template error: {}", e),
            TemplateError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for TemplateError {}

pub struct TemplateRenderer {
    tera: Tera,
    context: Context,
}

impl TemplateRenderer {
    pub fn new(theme_path: &str) -> Result<Self, TemplateError> {
        let tera = Tera::new(theme_path)?;
        let context = Context::new();
        
        Ok(Self { tera, context })
    }
    
    /// Add a value to the template context
    pub fn add_to_context<T: Serialize>(&mut self, key: &str, value: &T) {
        self.context.insert(key, value);
    }
    
    /// Get mutable access to the context for complex operations
    pub fn get_context_mut(&mut self) -> &mut Context {
        &mut self.context
    }
    
    /// Render a template with the current context
    pub fn render(&self, template: &str) -> Result<String, TemplateError> {
        Ok(self.tera.render(template, &self.context)?)
    }
    
    /// Render a template and write it directly to a file
    pub fn render_to_file(&self, template: &str, output_path: &Path) -> Result<(), TemplateError> {
        let rendered = self.render(template)?;
        
        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::write(output_path, rendered)?;
        Ok(())
    }
    
    /// Render a template with an external context (for new renderer)
    pub fn render_with_context(&self, template: &str, context: &Context) -> Result<String, TemplateError> {
        Ok(self.tera.render(template, context)?)
    }
    
    /// Render a template with an external context and write to file
    pub fn render_to_file_with_context(&self, template: &str, context: &Context, output_path: &Path) -> Result<(), TemplateError> {
        let rendered = self.tera.render(template, context)?;
        
        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::write(output_path, rendered)?;
        Ok(())
    }
}
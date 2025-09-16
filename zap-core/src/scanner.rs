use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::markdown::get_page_title;
use crate::site::{Page, PageType, Collection};

#[derive(Debug)]
pub enum ScanError {
    IoError(std::io::Error),
    InvalidPath(PathBuf),
}

impl From<std::io::Error> for ScanError {
    fn from(err: std::io::Error) -> Self {
        ScanError::IoError(err)
    }
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::IoError(e) => write!(f, "IO error: {}", e),
            ScanError::InvalidPath(p) => write!(f, "Invalid path: {}", p.display()),
        }
    }
}

impl std::error::Error for ScanError {}

pub struct SiteScanner {
    source_dir: PathBuf,
}

impl SiteScanner {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            source_dir: path.as_ref().to_path_buf(),
        }
    }

    pub fn scan(&self) -> Result<(Vec<Page>, Vec<Collection>), ScanError> {
        println!("Scanning: {}", self.source_dir.display());
        
        let pages = self.scan_pages()?;
        let collections = self.scan_collections()?;
        
        Ok((pages, collections))
    }

    pub fn scan_pages(&self) -> Result<Vec<Page>, ScanError> {
        let mut pages = Vec::new();
        
        for entry in std::fs::read_dir(&self.source_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            // Only process markdown files in the root directory
            if path.is_file() && get_extension(&path) == "md" {
                if let Some(page) = self.scan_page(path)? {
                    pages.push(page);
                }
            }
        }
        
        Ok(pages)
    }

    pub fn scan_collections(&self) -> Result<Vec<Collection>, ScanError> {
        let mut collections = Vec::new();
        
        for entry in std::fs::read_dir(&self.source_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            // Only process directories
            if path.is_dir() {
                let collection = self.scan_collection(path)?;
                collections.push(collection);
            }
        }
        
        Ok(collections)
    }

    fn scan_page(&self, path: PathBuf) -> Result<Option<Page>, ScanError> {
        let Some(file_name) = path.file_name() else {
            return Ok(None);
        };

        let page_type = match file_name.to_string_lossy().to_lowercase().as_str() {
            "readme.md" => PageType::Home,
            "changelog.md" => PageType::Changelog,
            "index.md" => PageType::Index,
            _ => PageType::Regular,
        };

        let title = get_page_title(&path);
        let relative_path = path.strip_prefix(&self.source_dir)
            .map_err(|_| ScanError::InvalidPath(path.clone()))?;

        Ok(Some(Page {
            title,
            path: relative_path.to_path_buf(),
            page_type,
        }))
    }

    fn scan_collection(&self, path: PathBuf) -> Result<Collection, ScanError> {
        let collection_name = path.file_name()
            .ok_or_else(|| ScanError::InvalidPath(path.clone()))?
            .to_string_lossy()
            .to_string();

        let mut collection = Collection {
            name: collection_name,
            pages: Vec::new(),
        };

        for markdown_file in get_all_markdown_files(&path) {
            if let Some(page) = self.scan_page(markdown_file)? {
                collection.pages.push(page);
            }
        }

        Ok(collection)
    }
}

fn get_all_markdown_files<P: AsRef<Path>>(path: P) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = Vec::new();
    for p in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|p| {
            p.path().is_file() && p.path().extension().map(|ext| ext == "md").unwrap_or(false)
        })
    {
        paths.push(p.path().to_path_buf());
    }

    paths
}

fn get_extension(path: &PathBuf) -> String {
    match path.extension() {
        Some(ext) => ext.to_string_lossy().to_string(),
        None => "Unknown".into(),
    }
}
use crate::markdown::{PageElement, get_page_structured, get_page_title};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct Page {
    pub title: String,
    pub path: PathBuf,
    pub page_type: PageType,
}

impl Page {
    pub fn elements(&self) -> Vec<PageElement> {
        println!("My path is: {}", &self.path.display());
        get_page_structured(&self.path)
    }

    pub fn template_name(&self) -> &'static str {
        match self.page_type {
            PageType::Home => "home.html",
            PageType::Changelog => "changelog.html",
            _ => "page.html",
        }
    }

    pub fn get_structured_elements(&self, source_dir: &Path) -> Vec<PageElement> {
        let full_path = source_dir.join(&self.path);
        get_page_structured(&full_path)
    }

    pub fn get_first_heading(&self, source_dir: &Path) -> Option<String> {
        self.get_structured_elements(source_dir)
            .into_iter()
            .find_map(|element| match element {
                PageElement::Heading { text, .. } => Some(text),
                _ => None,
            })
    }

    pub fn get_first_paragraph(&self, source_dir: &Path) -> Option<String> {
        self.get_structured_elements(source_dir)
            .into_iter()
            .find_map(|element| match element {
                PageElement::Paragraph { text } => Some(text),
                _ => None,
            })
    }
}

#[derive(Debug, Clone)]
pub enum PageType {
    Home,
    Changelog,
    Index,
    Regular,
    Unknown,
}

#[derive(Clone)]
pub struct Collection {
    pub name: String,
    pub pages: Vec<Page>,
}

impl Collection {
    pub fn url(&self) -> String {
        self.name.to_lowercase()
    }
}

pub struct Zap {
    pub scan_path: PathBuf,
    out_path: PathBuf,
    pages: Vec<Page>,
    collections: Vec<Collection>,
}

impl Zap {
    pub fn new(scan_path: PathBuf) -> Self {
        Self {
            scan_path,
            out_path: PathBuf::from("./out"),
            pages: Vec::new(),
            collections: Vec::new(),
        }
    }

    pub fn set_out_path(&mut self, path: PathBuf) {
        self.out_path = path;
    }

    pub fn scan(&mut self) {
        println!("Scanning: {}", &self.scan_path.display());
        for path in std::fs::read_dir(&self.scan_path)
            .expect("Failed to read scan path")
            .filter_map(|e| e.ok())
        {
            if path.path().is_dir() {
                self.collections.push(self.scan_collection(path.path()));
            } else if get_extension(path.path().to_path_buf()) == "md" {
                self.pages.push(self.scan_page(path.path()).unwrap());
            }
        }
    }

    fn scan_collection(&self, path: PathBuf) -> Collection {
        let mut collection = Collection {
            name: path.file_name().unwrap().to_string_lossy().to_string(),
            pages: Vec::new(),
        };

        for f in get_all_markdown_files(path) {
            collection.pages.push(self.scan_page(f).unwrap());
        }

        collection
    }

    fn scan_page(&self, path: PathBuf) -> Option<Page> {
        if path.file_name().is_none() {
            return None;
        }

        let page_type = match path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_lowercase()
            .as_str()
        {
            "readme.md" => PageType::Home,
            "changelog.md" => PageType::Changelog,
            "index.md" => PageType::Index,
            _ => PageType::Regular,
        };

        let title = get_page_title(&path);
        let relative_path = path.strip_prefix(&self.scan_path).unwrap();

        Some(Page {
            title,
            path: relative_path.to_path_buf(),
            page_type,
        })
    }

    pub fn pages(&self) -> &Vec<Page> {
        &self.pages
    }

    pub fn collections(&self) -> &Vec<Collection> {
        &self.collections
    }

    pub fn render_page(&self, page: &Page) -> String {
        crate::renderer::render_page(&self.scan_path, page)
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

fn get_extension(path: PathBuf) -> String {
    match path.extension() {
        Some(ext) => ext.to_string_lossy().to_string(),
        None => "Uknown".into(),
    }
}

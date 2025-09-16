pub mod config;
use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, html, CodeBlockKind};
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;
use walkdir::WalkDir;

// Initialize syntax highlighting resources once
static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(|| SyntaxSet::load_defaults_newlines());
static THEME_SET: LazyLock<ThemeSet> = LazyLock::new(|| ThemeSet::load_defaults());

pub fn parse_page(path: &str) -> Result<String, std::io::Error> {
    let content = std::fs::read_to_string(path)?;
    let options = Options::all();
    let parser = Parser::new_ext(&content, options);

    let events: Vec<Event> = parser.collect();
    let mut processed_events = Vec::new();
    let mut i = 0;

    while i < events.len() {
        match &events[i] {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                // Collect all text events until the end of the code block
                let mut code_content = String::new();
                i += 1; // Skip the start event
                
                while i < events.len() {
                    match &events[i] {
                        Event::End(TagEnd::CodeBlock) => break,
                        Event::Text(text) => code_content.push_str(text),
                        _ => {} // Ignore other events inside code blocks
                    }
                    i += 1;
                }

                // Generate syntax highlighted HTML
                let syntax = SYNTAX_SET.find_syntax_by_token(lang)
                    .or_else(|| {
                        // Fallback mappings for unsupported languages
                        match lang.as_ref() {
                            "nix" => SYNTAX_SET.find_syntax_by_name("JavaScript"), // Nix has similar structure
                            "toml" => SYNTAX_SET.find_syntax_by_name("YAML"), // TOML similar to YAML
                            _ => None
                        }
                    });

                let highlighted_html = if let Some(syntax) = syntax {
                    let theme = &THEME_SET.themes["base16-ocean.dark"];
                    highlighted_html_for_string(&code_content, &SYNTAX_SET, syntax, theme)
                        .unwrap_or_else(|_| format!("<pre><code>{}</code></pre>", html_escape::encode_text(&code_content)))
                } else {
                    println!("No syntax found for language: {}", lang);
                    format!("<pre><code>{}</code></pre>", html_escape::encode_text(&code_content))
                };

                processed_events.push(Event::Html(highlighted_html.into()));
            }
            _ => {
                processed_events.push(events[i].clone());
            }
        }
        i += 1;
    }

    let mut out = String::new();
    html::push_html(&mut out, processed_events.into_iter());

    Ok(out)
}

#[derive(Debug)]
pub struct Page {
    pub title: String,
    pub path: PathBuf,
    pub page_type: PageType,
}

impl Page {
    pub fn url(&self) -> String {
        self.out_path()
            .with_file_name("")
            .to_string_lossy()
            .to_string()
    }

    pub fn out_path(&self) -> PathBuf {
        let out: PathBuf = PathBuf::new();

        match &self.page_type {
            PageType::Home => out.join("index.html"),
            PageType::Changelog => out.join("changelog/index.html"),
            PageType::Index => out
                .join(&self.path.with_file_name("").with_extension(""))
                .join("index.html"),
            _ => out.join(&self.path.with_extension("")).join("index.html"),
        }
    }
}

#[derive(Debug)]
pub enum PageType {
    Home,
    Changelog,
    Index,
    Regular,
    Unknown,
}

pub struct Collection {
    pub name: String,
    pub pages: Vec<Page>,
}

impl Collection {
    pub fn url(&self) -> String {
        self.name.to_lowercase()
    }
}
struct Heading {
    level: u32,
    text: String,
}

fn get_page_headings(path: &PathBuf) -> Vec<Heading> {
    let content = std::fs::read_to_string(path).expect("Faild to rd some page sry");
    let options = Options::all();
    let parser = Parser::new_ext(&content, options);

    let mut in_heading = false;
    let mut lvl: u32 = 0;
    let mut text_buf = String::new();
    let mut headings: Vec<Heading> = Vec::new();
    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                in_heading = true;
                lvl = level as u32;
            }
            Event::End(TagEnd::Heading { .. }) => {
                if in_heading {
                    headings.push(Heading {
                        level: lvl,
                        text: text_buf.to_owned(),
                    });
                    text_buf.clear();
                    in_heading = false;
                }
            }
            Event::Text(text) => {
                if in_heading {
                    text_buf.push_str(&text)
                }
            }
            _ => continue,
        };
    }

    headings
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

pub struct Zap {
    scan_path: PathBuf,
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

    pub fn render_page(&self, page: &Page) -> String {
        let page_path = self.scan_path.join(&page.path);

        match parse_page(page_path.to_string_lossy().to_string().as_str()) {
            Ok(out) => out,
            Err(_) => String::from("Sadly failed"),
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

        let title = match get_page_headings(&path).first() {
            Some(h) => h.text.to_owned(),
            None => "A sad page".to_string(),
        };

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
}

fn get_extension(path: PathBuf) -> String {
    match path.extension() {
        Some(ext) => ext.to_string_lossy().to_string(),
        None => "Uknown".into(),
    }
}

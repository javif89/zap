use std::sync::LazyLock;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, html, CodeBlockKind};
use syntect::highlighting::ThemeSet;
use syntect::html::highlighted_html_for_string;
use syntect::parsing::SyntaxSet;

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

#[derive(Debug, Clone)]
pub enum PageElement {
    Heading { level: u32, text: String },
    Paragraph { text: String },
    CodeBlock { language: Option<String>, content: String },
    List { items: Vec<String>, ordered: bool },
    BlockQuote { text: String },
}

struct Heading {
    level: u32,
    text: String,
}

fn get_page_headings(path: &std::path::PathBuf) -> Vec<Heading> {
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

pub fn get_page_structured(path: &std::path::PathBuf) -> Vec<PageElement> {
    let content = std::fs::read_to_string(path).expect("Failed to read page");
    let options = Options::all();
    let parser = Parser::new_ext(&content, options);

    let mut elements = Vec::new();
    let mut current_element: Option<PageElement> = None;
    let mut text_buf = String::new();
    let mut list_items = Vec::new();
    let mut current_level = 0;
    let mut current_ordered = false;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                current_level = level as u32;
                text_buf.clear();
            }
            Event::End(TagEnd::Heading { .. }) => {
                if !text_buf.is_empty() {
                    elements.push(PageElement::Heading {
                        level: current_level,
                        text: text_buf.trim().to_string(),
                    });
                    text_buf.clear();
                }
            }
            Event::Start(Tag::Paragraph) => {
                text_buf.clear();
            }
            Event::End(TagEnd::Paragraph) => {
                if !text_buf.is_empty() {
                    elements.push(PageElement::Paragraph {
                        text: text_buf.trim().to_string(),
                    });
                    text_buf.clear();
                }
            }
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                text_buf.clear();
                current_element = Some(PageElement::CodeBlock {
                    language: if lang.is_empty() { None } else { Some(lang.to_string()) },
                    content: String::new(),
                });
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some(PageElement::CodeBlock { language, .. }) = current_element.take() {
                    elements.push(PageElement::CodeBlock {
                        language,
                        content: text_buf.trim().to_string(),
                    });
                    text_buf.clear();
                }
            }
            Event::Start(Tag::List(start_num)) => {
                current_ordered = start_num.is_some();
                list_items.clear();
                text_buf.clear();
            }
            Event::End(TagEnd::List(_)) => {
                if !list_items.is_empty() {
                    elements.push(PageElement::List {
                        items: list_items.clone(),
                        ordered: current_ordered,
                    });
                    list_items.clear();
                }
            }
            Event::Start(Tag::Item) => {
                text_buf.clear();
            }
            Event::End(TagEnd::Item) => {
                if !text_buf.is_empty() {
                    list_items.push(text_buf.trim().to_string());
                    text_buf.clear();
                }
            }
            Event::Start(Tag::BlockQuote(_)) => {
                text_buf.clear();
            }
            Event::End(TagEnd::BlockQuote(_)) => {
                if !text_buf.is_empty() {
                    elements.push(PageElement::BlockQuote {
                        text: text_buf.trim().to_string(),
                    });
                    text_buf.clear();
                }
            }
            Event::Text(text) => {
                text_buf.push_str(&text);
            }
            Event::SoftBreak | Event::HardBreak => {
                text_buf.push(' ');
            }
            _ => {}
        }
    }

    elements
}

pub fn get_page_title(path: &std::path::PathBuf) -> String {
    match get_page_headings(path).first() {
        Some(h) => h.text.to_owned(),
        None => "A sad page".to_string(),
    }
}
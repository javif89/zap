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
    Heading { level: u32, content: Vec<InlineElement> },
    Paragraph { content: Vec<InlineElement> },
    CodeBlock { language: Option<String>, content: String },
    List { items: Vec<ListItem>, ordered: bool },
    BlockQuote { content: Vec<PageElement> },
    Table { headers: Vec<Vec<InlineElement>>, rows: Vec<Vec<Vec<InlineElement>>> },
    HorizontalRule,
    Html { content: String },
}

#[derive(Debug, Clone)]
pub enum InlineElement {
    Text(String),
    Link { text: String, url: String, title: Option<String> },
    Image { alt: String, url: String, title: Option<String> },
    Emphasis { level: u8, content: Vec<InlineElement> }, // 1=italic, 2=bold
    Code(String),
    SoftBreak,
    HardBreak,
    Strikethrough { content: Vec<InlineElement> },
}

#[derive(Debug, Clone)]
pub struct ListItem {
    pub content: Vec<InlineElement>,
    pub sub_items: Vec<ListItem>,
    pub checked: Option<bool>, // For task lists
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
    let mut stack: Vec<ElementBuilder> = Vec::new();
    
    for event in parser {
        match event {
            Event::Start(tag) => {
                stack.push(ElementBuilder::from_tag(tag));
            }
            Event::End(_) => {
                if let Some(builder) = stack.pop() {
                    // Special handling for list items
                    if matches!(builder.kind, BuilderKind::ListItem(_)) {
                        // List items should add their content to the parent list
                        if let Some(parent) = stack.last_mut() {
                            if matches!(parent.kind, BuilderKind::List(_)) {
                                parent.list_items.push(ListItem {
                                    content: builder.inline_content,
                                    sub_items: Vec::new(),
                                    checked: None,
                                });
                            }
                        }
                    } else if matches!(builder.kind, BuilderKind::Emphasis(_) | BuilderKind::Strikethrough | BuilderKind::Link(_, _) | BuilderKind::Image(_, _)) {
                        // Inline elements should be added to the parent's inline content
                        if let Some(parent) = stack.last_mut() {
                            match builder.kind {
                                BuilderKind::Emphasis(level) => {
                                    parent.add_inline(InlineElement::Emphasis {
                                        level,
                                        content: builder.inline_content,
                                    });
                                }
                                BuilderKind::Strikethrough => {
                                    parent.add_inline(InlineElement::Strikethrough {
                                        content: builder.inline_content,
                                    });
                                }
                                BuilderKind::Link(url, title) => {
                                    let text = render_inline_elements_text(&builder.inline_content);
                                    parent.add_inline(InlineElement::Link { text, url, title });
                                }
                                BuilderKind::Image(url, title) => {
                                    let alt = render_inline_elements_text(&builder.inline_content);
                                    parent.add_inline(InlineElement::Image { alt, url, title });
                                }
                                _ => {}
                            }
                        }
                    } else {
                        let element = builder.build();
                        if stack.is_empty() {
                            if let Some(elem) = element {
                                elements.push(elem);
                            }
                        } else if let Some(parent) = stack.last_mut() {
                            parent.add_child(element);
                        }
                    }
                }
            }
            Event::Text(text) => {
                if let Some(builder) = stack.last_mut() {
                    builder.add_inline(InlineElement::Text(text.to_string()));
                }
            }
            Event::Code(code) => {
                if let Some(builder) = stack.last_mut() {
                    builder.add_inline(InlineElement::Code(code.to_string()));
                }
            }
            Event::SoftBreak => {
                if let Some(builder) = stack.last_mut() {
                    builder.add_inline(InlineElement::SoftBreak);
                }
            }
            Event::HardBreak => {
                if let Some(builder) = stack.last_mut() {
                    builder.add_inline(InlineElement::HardBreak);
                }
            }
            Event::Rule => {
                elements.push(PageElement::HorizontalRule);
            }
            Event::Html(html) => {
                elements.push(PageElement::Html { content: html.to_string() });
            }
            _ => {}
        }
    }
    
    elements
}

#[derive(Debug)]
struct ElementBuilder {
    kind: BuilderKind,
    inline_content: Vec<InlineElement>,
    block_content: Vec<PageElement>,
    list_items: Vec<ListItem>,
    table_data: TableBuilder,
}

#[derive(Debug)]
enum BuilderKind {
    Heading(u32),
    Paragraph,
    CodeBlock(Option<String>),
    List(bool), // ordered
    BlockQuote,
    ListItem(Option<bool>), // checked state for task lists
    Table,
    TableHead,
    TableRow,
    TableCell,
    Emphasis(u8),
    Strikethrough,
    Link(String, Option<String>), // url, title
    Image(String, Option<String>), // url, title
}

#[derive(Debug, Default)]
struct TableBuilder {
    headers: Vec<Vec<InlineElement>>,
    rows: Vec<Vec<Vec<InlineElement>>>,
    current_row: Vec<Vec<InlineElement>>,
}

impl ElementBuilder {
    fn from_tag(tag: Tag) -> Self {
        let kind = match tag {
            Tag::Heading { level, .. } => BuilderKind::Heading(level as u32),
            Tag::Paragraph => BuilderKind::Paragraph,
            Tag::CodeBlock(CodeBlockKind::Fenced(lang)) => {
                BuilderKind::CodeBlock(if lang.is_empty() { None } else { Some(lang.to_string()) })
            }
            Tag::CodeBlock(CodeBlockKind::Indented) => BuilderKind::CodeBlock(None),
            Tag::List(start) => BuilderKind::List(start.is_some()),
            Tag::Item => BuilderKind::ListItem(None),
            Tag::BlockQuote(_) => BuilderKind::BlockQuote,
            Tag::Table(_) => BuilderKind::Table,
            Tag::TableHead => BuilderKind::TableHead,
            Tag::TableRow => BuilderKind::TableRow,
            Tag::TableCell => BuilderKind::TableCell,
            Tag::Emphasis => BuilderKind::Emphasis(1),
            Tag::Strong => BuilderKind::Emphasis(2),
            Tag::Strikethrough => BuilderKind::Strikethrough,
            Tag::Link { dest_url, title, .. } => {
                let title_str = if title.is_empty() { None } else { Some(title.to_string()) };
                BuilderKind::Link(dest_url.to_string(), title_str)
            }
            Tag::Image { dest_url, title, .. } => {
                let title_str = if title.is_empty() { None } else { Some(title.to_string()) };
                BuilderKind::Image(dest_url.to_string(), title_str)
            }
            _ => BuilderKind::Paragraph, // Fallback
        };
        
        Self {
            kind,
            inline_content: Vec::new(),
            block_content: Vec::new(),
            list_items: Vec::new(),
            table_data: TableBuilder::default(),
        }
    }
    
    fn add_inline(&mut self, elem: InlineElement) {
        self.inline_content.push(elem);
    }
    
    fn add_child(&mut self, child: Option<PageElement>) {
        if let Some(elem) = child {
            match &mut self.kind {
                BuilderKind::BlockQuote => {
                    self.block_content.push(elem);
                }
                BuilderKind::List(_) => {
                    // List items are handled differently - they return a special ListItem element
                    // that gets added to our list_items vec
                }
                _ => {}
            }
        }
    }
    
    fn build(self) -> Option<PageElement> {
        match self.kind {
            BuilderKind::Heading(level) => {
                Some(PageElement::Heading {
                    level,
                    content: self.inline_content,
                })
            }
            BuilderKind::Paragraph => {
                if !self.inline_content.is_empty() {
                    Some(PageElement::Paragraph {
                        content: self.inline_content,
                    })
                } else {
                    None
                }
            }
            BuilderKind::CodeBlock(language) => {
                let content = self.inline_content.iter()
                    .map(|e| match e {
                        InlineElement::Text(s) => s.clone(),
                        _ => String::new(),
                    })
                    .collect::<String>();
                Some(PageElement::CodeBlock { language, content })
            }
            BuilderKind::List(ordered) => {
                Some(PageElement::List {
                    items: self.list_items,
                    ordered,
                })
            }
            BuilderKind::BlockQuote => {
                Some(PageElement::BlockQuote {
                    content: self.block_content,
                })
            }
            BuilderKind::ListItem(_) => {
                // List items should be handled by their parent List
                // We return None here, but the List builder should collect the inline content
                None
            }
            _ => None,
        }
    }
}

pub fn get_page_title(path: &std::path::PathBuf) -> String {
    match get_page_headings(path).first() {
        Some(h) => h.text.to_owned(),
        None => "A sad page".to_string(),
    }
}

// HTML Rendering functions
pub fn render_elements_to_html(elements: &[PageElement]) -> String {
    let mut html = String::new();
    
    for element in elements {
        html.push_str(&render_element(element));
    }
    
    html
}

fn render_element(element: &PageElement) -> String {
    match element {
        PageElement::Heading { level, content } => {
            format!("<h{0}>{1}</h{0}>\n", level, render_inline_elements(content))
        }
        PageElement::Paragraph { content } => {
            format!("<p>{}</p>\n", render_inline_elements(content))
        }
        PageElement::CodeBlock { language, content } => {
            if let Some(lang) = language {
                // Use syntect for highlighting
                let syntax = SYNTAX_SET.find_syntax_by_token(lang)
                    .or_else(|| {
                        match lang.as_str() {
                            "nix" => SYNTAX_SET.find_syntax_by_name("JavaScript"),
                            "toml" => SYNTAX_SET.find_syntax_by_name("YAML"),
                            _ => None
                        }
                    });
                
                if let Some(syntax) = syntax {
                    let theme = &THEME_SET.themes["base16-ocean.dark"];
                    highlighted_html_for_string(content, &SYNTAX_SET, syntax, theme)
                        .unwrap_or_else(|_| format!("<pre><code>{}</code></pre>\n", html_escape::encode_text(content)))
                } else {
                    format!("<pre><code>{}</code></pre>\n", html_escape::encode_text(content))
                }
            } else {
                format!("<pre><code>{}</code></pre>\n", html_escape::encode_text(content))
            }
        }
        PageElement::List { items, ordered } => {
            let tag = if *ordered { "ol" } else { "ul" };
            let items_html: String = items.iter()
                .map(|item| render_list_item(item))
                .collect();
            format!("<{0}>\n{1}</{0}>\n", tag, items_html)
        }
        PageElement::BlockQuote { content } => {
            let inner = render_elements_to_html(content);
            format!("<blockquote>\n{}</blockquote>\n", inner)
        }
        PageElement::Table { headers, rows } => {
            render_table(headers, rows)
        }
        PageElement::HorizontalRule => "<hr />\n".to_string(),
        PageElement::Html { content } => format!("{}\n", content),
    }
}

pub fn render_inline_elements_text(elements: &[InlineElement]) -> String {
    let mut text = String::new();
    
    for element in elements {
        match element {
            InlineElement::Text(s) => text.push_str(s),
            InlineElement::Link { text: link_text, .. } => text.push_str(link_text),
            InlineElement::Image { alt, .. } => text.push_str(alt),
            InlineElement::Emphasis { content, .. } => {
                text.push_str(&render_inline_elements_text(content));
            }
            InlineElement::Code(code) => text.push_str(code),
            InlineElement::SoftBreak | InlineElement::HardBreak => text.push(' '),
            InlineElement::Strikethrough { content } => {
                text.push_str(&render_inline_elements_text(content));
            }
        }
    }
    
    text
}

fn render_inline_elements(elements: &[InlineElement]) -> String {
    let mut html = String::new();
    
    for element in elements {
        match element {
            InlineElement::Text(text) => {
                html.push_str(&html_escape::encode_text(text));
            }
            InlineElement::Link { text, url, title } => {
                let title_attr = title.as_ref()
                    .map(|t| format!(" title=\"{}\"", html_escape::encode_quoted_attribute(t)))
                    .unwrap_or_default();
                html.push_str(&format!("<a href=\"{}\"{}>{}</a>", 
                    html_escape::encode_quoted_attribute(url),
                    title_attr,
                    html_escape::encode_text(text)
                ));
            }
            InlineElement::Image { alt, url, title } => {
                let title_attr = title.as_ref()
                    .map(|t| format!(" title=\"{}\"", html_escape::encode_quoted_attribute(t)))
                    .unwrap_or_default();
                html.push_str(&format!("<img src=\"{}\" alt=\"{}\"{}/>", 
                    html_escape::encode_quoted_attribute(url),
                    html_escape::encode_quoted_attribute(alt),
                    title_attr
                ));
            }
            InlineElement::Emphasis { level, content } => {
                match level {
                    1 => html.push_str(&format!("<em>{}</em>", render_inline_elements(content))),
                    2 => html.push_str(&format!("<strong>{}</strong>", render_inline_elements(content))),
                    _ => html.push_str(&render_inline_elements(content)),
                }
            }
            InlineElement::Code(code) => {
                html.push_str(&format!("<code>{}</code>", html_escape::encode_text(code)));
            }
            InlineElement::SoftBreak => html.push(' '),
            InlineElement::HardBreak => html.push_str("<br />"),
            InlineElement::Strikethrough { content } => {
                html.push_str(&format!("<del>{}</del>", render_inline_elements(content)));
            }
        }
    }
    
    html
}

fn render_list_item(item: &ListItem) -> String {
    let mut html = String::new();
    
    if let Some(checked) = item.checked {
        let checkbox = if checked {
            "<input type=\"checkbox\" checked disabled/> "
        } else {
            "<input type=\"checkbox\" disabled/> "
        };
        html.push_str(&format!("<li>{}{}</li>\n", checkbox, render_inline_elements(&item.content)));
    } else {
        html.push_str(&format!("<li>{}", render_inline_elements(&item.content)));
        
        if !item.sub_items.is_empty() {
            html.push_str("\n<ul>\n");
            for sub_item in &item.sub_items {
                html.push_str(&render_list_item(sub_item));
            }
            html.push_str("</ul>\n");
        }
        
        html.push_str("</li>\n");
    }
    
    html
}

fn render_table(headers: &[Vec<InlineElement>], rows: &[Vec<Vec<InlineElement>>]) -> String {
    let mut html = String::from("<table>\n");
    
    if !headers.is_empty() {
        html.push_str("<thead>\n<tr>\n");
        for header in headers {
            html.push_str(&format!("<th>{}</th>\n", render_inline_elements(header)));
        }
        html.push_str("</tr>\n</thead>\n");
    }
    
    if !rows.is_empty() {
        html.push_str("<tbody>\n");
        for row in rows {
            html.push_str("<tr>\n");
            for cell in row {
                html.push_str(&format!("<td>{}</td>\n", render_inline_elements(cell)));
            }
            html.push_str("</tr>\n");
        }
        html.push_str("</tbody>\n");
    }
    
    html.push_str("</table>\n");
    html
}
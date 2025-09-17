use anyhow::Result;
use axum::{
    Router,
    extract::State,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
};
use clap::{Arg, ArgMatches, Command};
use notify_debouncer_mini::{DebounceEventResult, new_debouncer};
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::sync::broadcast;
use tower_http::services::ServeDir;
use zap_core::{NavItem, PageType, SiteBuilder, SiteScanner};
use crate::config::load_serve_config;

pub fn make_subcommand() -> Command {
    Command::new("serve")
        .about("Start development server with live reload")
        .arg(
            Arg::new("source")
                .short('s')
                .long("source")
                .value_name("DIR")
                .help("Source directory containing markdown files")
                .default_value("./site"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("DIR")
                .help("Output directory for generated site")
                .default_value("./out"),
        )
        .arg(
            Arg::new("theme")
                .short('t')
                .long("theme")
                .value_name("DIR")
                .help("Theme directory")
                .default_value("./theme"),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Configuration file")
                .default_value("./zap.toml"),
        )
        .arg(
            Arg::new("port")
                .short('p')
                .long("port")
                .value_name("PORT")
                .help("Port to serve on")
                .default_value("3000"),
        )
        .arg(
            Arg::new("host")
                .long("host")
                .value_name("HOST")
                .help("Host to bind to")
                .default_value("127.0.0.1"),
        )
        .arg(
            Arg::new("open")
                .long("open")
                .help("Open browser automatically")
                .action(clap::ArgAction::SetTrue),
        )
}

#[derive(Clone)]
struct AppState {
    reload_tx: broadcast::Sender<String>,
    source_dir: PathBuf,
    output_dir: PathBuf,
    theme_dir: PathBuf,
    config_file: PathBuf,
    livereload_host: String,
    zap_config: crate::config::ZapConfig,
}

pub async fn execute(args: &ArgMatches) -> Result<()> {
    // Load cascading configuration
    let zap_config = load_serve_config(args)?;
    let build_config = zap_config.build_config();

    let source_dir = PathBuf::from(&build_config.source);
    let output_dir = PathBuf::from(&build_config.output);
    let theme_dir = PathBuf::from(&build_config.theme);
    let config_file = PathBuf::from(&build_config.config);
    let port = build_config.port;
    let host = &build_config.host;
    let open_browser = build_config.open;

    // Initial build with livereload
    let livereload_host = format!("{}:{}", host, port);
    build_site_with_livereload(
        &source_dir,
        &output_dir,
        &theme_dir,
        &config_file,
        Some(&livereload_host),
        &zap_config,
    )?;

    // Create broadcast channel for live reload
    let (reload_tx, _) = broadcast::channel::<String>(100);

    let state = AppState {
        reload_tx: reload_tx.clone(),
        source_dir: source_dir.clone(),
        output_dir: output_dir.clone(),
        theme_dir: theme_dir.clone(),
        config_file: config_file.clone(),
        livereload_host: livereload_host.clone(),
        zap_config: zap_config.clone(),
    };

    // Start file watcher
    let watcher_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = start_file_watcher(watcher_state).await {
            eprintln!("File watcher error: {}", e);
        }
    });

    // Create router
    let serve_dir = ServeDir::new(&output_dir);
    let app = Router::new()
        .route("/__livereload", get(websocket_handler))
        .nest_service("/", serve_dir)
        .with_state(state);

    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;

    println!("Serving site at http://{}", addr);
    println!("Serving files from: {}", output_dir.display());
    println!("Watching for changes in: {}", source_dir.display());

    if open_browser
        && let Err(e) = open::that(format!("http://{}", addr)) {
            eprintln!("Failed to open browser: {}", e);
        }

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket_connection(socket, state.reload_tx))
}

async fn websocket_connection(mut socket: WebSocket, reload_tx: broadcast::Sender<String>) {
    let mut rx = reload_tx.subscribe();

    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Ok(reload_msg) => {
                        if socket.send(Message::Text(reload_msg)).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            msg = socket.recv() => {
                if msg.is_none() {
                    break;
                }
            }
        }
    }
}

async fn start_file_watcher(state: AppState) -> Result<()> {
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    let mut debouncer = new_debouncer(
        Duration::from_millis(250),
        move |res: DebounceEventResult| {
            if let Ok(events) = res {
                for event in events {
                    let _ = tx.blocking_send(event.path);
                }
            }
        },
    )?;

    // Watch source directory
    debouncer
        .watcher()
        .watch(&state.source_dir, notify::RecursiveMode::Recursive)?;

    // Watch theme directory if it exists
    if state.theme_dir.exists() {
        debouncer
            .watcher()
            .watch(&state.theme_dir, notify::RecursiveMode::Recursive)?;
    }

    // Watch config file if it exists
    if state.config_file.exists() {
        debouncer
            .watcher()
            .watch(&state.config_file, notify::RecursiveMode::NonRecursive)?;
    }

    println!("File watcher started");

    while let Some(path) = rx.recv().await {
        println!("File changed: {}", path.display());

        // Rebuild site with livereload
        match build_site_with_livereload(
            &state.source_dir,
            &state.output_dir,
            &state.theme_dir,
            &state.config_file,
            Some(&state.livereload_host),
            &state.zap_config,
        ) {
            Ok(_) => {
                println!("Site rebuilt successfully");
                // Send reload message to all connected clients
                let _ = state.reload_tx.send("reload".to_string());
            }
            Err(e) => {
                eprintln!("Build error: {}", e);
            }
        }
    }

    Ok(())
}

fn build_site(
    source_dir: &Path,
    output_dir: &Path,
    theme_dir: &Path,
    config_file: &Path,
    zap_config: &crate::config::ZapConfig,
) -> Result<()> {
    build_site_with_livereload(source_dir, output_dir, theme_dir, config_file, None, zap_config)
}

fn build_site_with_livereload(
    source_dir: &Path,
    output_dir: &Path,
    theme_dir: &Path,
    config_file: &Path,
    livereload_host: Option<&str>,
    zap_config: &crate::config::ZapConfig,
) -> Result<()> {
    // Use the cascading configuration
    let config = zap_config.site_config();

    let scanner = SiteScanner::new(source_dir);
    let (pages, collections) = scanner.scan()?;

    let mut navigation: Vec<NavItem> = pages
        .iter()
        .filter_map(|p| match p.page_type {
            PageType::Home => None,
            PageType::Changelog => None,
            _ => Some(NavItem {
                text: p.title.clone(),
                link: p.url(source_dir),
            }),
        })
        .collect();

    let collection_links: Vec<NavItem> = collections
        .iter()
        .map(|c| NavItem {
            text: crate::cmd::build::title_case(&c.name),
            link: format!("/{}", c.url()),
        })
        .collect();

    navigation.extend(collection_links);

    let home_config = config.home.clone().unwrap_or_default();
    let mut site_config = config.site.clone().unwrap_or_default();
    let home_page = pages.iter().find(|p| matches!(p.page_type, PageType::Home));

    if site_config.title.is_none() {
        site_config.title = home_page
            .and_then(|home| home.get_first_heading())
            .or_else(|| Some("Zap".to_string()));
    }

    // Only use README first paragraph as tagline if none is set in config
    if site_config.tagline.is_none() {
        site_config.tagline = home_page.and_then(|home| home.get_first_paragraph());
    }

    let mut builder = SiteBuilder::new()
        .source_dir(source_dir)
        .output_dir(output_dir)
        .theme_dir(theme_dir)
        .site_config(site_config)
        .home_config(home_config)
        .navigation(navigation);

    // Add livereload context if in development mode
    if let Some(host) = livereload_host {
        builder = builder.add_custom("livereload", host)?;
    }

    for page in pages {
        builder = builder.add_page(page);
    }
    for collection in collections {
        builder = builder.add_collection(collection);
    }

    let site = builder.build()?;
    site.render_all()?;

    Ok(())
}



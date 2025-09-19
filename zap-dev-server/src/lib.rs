use anyhow::Result;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
    routing::get,
    Router,
};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult};
use std::{
    net::SocketAddr,
    path::PathBuf,
    time::Duration,
};
use tokio::sync::broadcast;
use tower_http::services::ServeDir;

/// Configuration for the live development server
#[derive(Debug, Clone)]
pub struct LiveServerConfig {
    /// Host to bind to
    pub host: String,
    /// Port to serve on
    pub port: u16,
    /// Root directory to serve and watch
    pub root: PathBuf,
    /// Auto-open browser
    pub open: bool,
    /// Patterns to ignore when watching
    pub ignore: Vec<String>,
}

impl Default for LiveServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            root: PathBuf::from("."),
            open: false,
            ignore: vec![],
        }
    }
}

/// A live-reload static file server
pub struct LiveServer {
    config: LiveServerConfig,
}

impl LiveServer {
    /// Create a new live server with the given configuration
    pub fn new(config: LiveServerConfig) -> Self {
        Self { config }
    }

    /// Run the live server
    pub async fn run(self) -> Result<()> {
        // Create broadcast channel for live reload
        let (reload_tx, _) = broadcast::channel::<String>(100);

        // Ensure root directory exists
        if !self.config.root.exists() {
            return Err(anyhow::anyhow!(
                "Root directory does not exist: {}",
                self.config.root.display()
            ));
        }

        let state = AppState {
            reload_tx: reload_tx.clone(),
        };

        // Start file watcher
        let watcher_reload_tx = reload_tx.clone();
        let watch_path = self.config.root.clone();
        let ignore_patterns = self.config.ignore.clone();
        
        tokio::spawn(async move {
            if let Err(e) = start_file_watcher(watch_path, watcher_reload_tx, ignore_patterns).await {
                eprintln!("File watcher error: {}", e);
            }
        });

        // Create router
        let serve_dir = ServeDir::new(&self.config.root);
        let app = Router::new()
            .route("/__livereload", get(websocket_handler))
            .fallback_service(serve_dir)
            .with_state(state);

        // Build address
        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port).parse()?;

        println!("Serving at http://{}", addr);
        println!("Watching: {}", self.config.root.display());
        println!("Live reload enabled at ws://{}/__livereload", addr);

        // Open browser if requested
        if self.config.open {
            if let Err(e) = open::that(format!("http://{}", addr)) {
                eprintln!("Failed to open browser: {}", e);
            }
        }

        // Start server
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

#[derive(Clone)]
struct AppState {
    reload_tx: broadcast::Sender<String>,
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket_connection(socket, state.reload_tx))
}

async fn websocket_connection(mut socket: WebSocket, reload_tx: broadcast::Sender<String>) {
    let mut rx = reload_tx.subscribe();

    // Send initial connection confirmation
    if socket
        .send(Message::Text("connected".to_string().into()))
        .await
        .is_err()
    {
        return;
    }

    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Ok(reload_msg) => {
                        if socket.send(Message::Text(reload_msg.into())).await.is_err() {
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

async fn start_file_watcher(
    watch_path: PathBuf,
    reload_tx: broadcast::Sender<String>,
    ignore_patterns: Vec<String>,
) -> Result<()> {
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    let mut debouncer = new_debouncer(
        Duration::from_millis(500), // Increase debounce time
        move |res: DebounceEventResult| {
            if let Ok(events) = res {
                for event in events {
                    // Check if path should be ignored
                    let path_str = event.path.to_string_lossy();
                    let should_ignore = ignore_patterns
                        .iter()
                        .any(|pattern| path_str.contains(pattern));
                    
                    if !should_ignore {
                        let _ = tx.blocking_send(event.path);
                    }
                }
            }
        },
    )?;

    // Watch the root directory
    debouncer
        .watcher()
        .watch(&watch_path, notify::RecursiveMode::Recursive)?;

    println!("File watcher started for: {}", watch_path.display());

    // Process file change events with simple deduplication
    let mut last_reload = std::time::Instant::now();
    while let Some(path) = rx.recv().await {
        println!("File changed: {}", path.display());
        
        // Only send reload if enough time has passed since last reload
        let now = std::time::Instant::now();
        if now.duration_since(last_reload) > Duration::from_millis(1000) {
            // Send reload message to all connected clients
            let _ = reload_tx.send("reload".to_string());
            last_reload = now;
            println!("Sent reload signal");
        } else {
            println!("Skipping reload (too soon)");
        }
    }

    Ok(())
}

/// Inject live reload script into HTML content
pub fn inject_livereload_script(html: &str, host: &str, port: u16) -> String {
    let script = format!(
        r#"
<script>
(function() {{
    const socket = new WebSocket('ws://{}:{}/__livereload');
    socket.onmessage = function(event) {{
        if (event.data === 'reload') {{
            location.reload();
        }}
    }};
    socket.onclose = function() {{
        console.log('Live reload disconnected');
    }};
}})();
</script>
"#,
        host, port
    );

    // Try to inject before closing body tag, or at the end if not found
    if let Some(pos) = html.rfind("</body>") {
        let mut result = String::with_capacity(html.len() + script.len());
        result.push_str(&html[..pos]);
        result.push_str(&script);
        result.push_str(&html[pos..]);
        result
    } else {
        format!("{}{}", html, script)
    }
}

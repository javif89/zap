use anyhow::Result;
use clap::{Arg, ArgMatches, Command};
use notify_debouncer_mini::{DebounceEventResult, new_debouncer};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use zap_core::build_site;
use zap_dev_server::{LiveServer, LiveServerConfig};
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


pub async fn execute(args: &ArgMatches) -> Result<()> {
    // Load cascading configuration
    let zap_config = load_serve_config(args)?;
    let build_config = zap_config.build_config();

    let source_dir = PathBuf::from(&build_config.source);
    let output_dir = PathBuf::from(&build_config.output);
    let theme_dir = PathBuf::from(&build_config.theme);
    let config_file = PathBuf::from(&build_config.config);
    
    // Initial build with livereload support
    let livereload_host = format!("{}:{}", build_config.host, build_config.port);
    build_site_with_livereload(
        &zap_config,
        &source_dir,
        &output_dir,
        &theme_dir,
        &livereload_host,
    )?;

    // Start the live dev server (handles its own file watching of output dir)
    let server_config = LiveServerConfig {
        host: build_config.host.clone(),
        port: build_config.port,
        root: output_dir.clone(),
        open: build_config.open,
        ignore: vec![".git".to_string(), "*.tmp".to_string()],
    };
    
    let server = LiveServer::new(server_config);
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.run().await {
            eprintln!("Dev server error: {}", e);
        }
    });

    // Watch source files and rebuild on changes
    let watcher_config = zap_config.clone();
    let watcher_handle = tokio::spawn(async move {
        if let Err(e) = watch_source_files(watcher_config).await {
            eprintln!("Source watcher error: {}", e);
        }
    });

    // Wait for both tasks
    let _ = tokio::try_join!(server_handle, watcher_handle)?;

    Ok(())
}

async fn watch_source_files(config: crate::config::ZapConfig) -> Result<()> {
    let build_config = config.build_config();
    let source_dir = PathBuf::from(&build_config.source);
    let output_dir = PathBuf::from(&build_config.output);
    let theme_dir = PathBuf::from(&build_config.theme);
    let config_file = PathBuf::from(&build_config.config);
    let livereload_host = format!("{}:{}", build_config.host, build_config.port);
    
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    let mut debouncer = new_debouncer(
        Duration::from_millis(500), // Slightly longer delay for rebuilds
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
        .watch(&source_dir, notify::RecursiveMode::Recursive)?;
    println!("Watching source directory: {}", source_dir.display());

    // Watch theme directory if it exists
    if theme_dir.exists() {
        debouncer
            .watcher()
            .watch(&theme_dir, notify::RecursiveMode::Recursive)?;
        println!("Watching theme directory: {}", theme_dir.display());
    }

    // Watch config file if it exists
    if config_file.exists() {
        debouncer
            .watcher()
            .watch(&config_file, notify::RecursiveMode::NonRecursive)?;
        println!("Watching config file: {}", config_file.display());
    }

    println!("Watching source files for changes...");

    while let Some(path) = rx.recv().await {
        println!("Source file changed: {} (absolute: {})", path.display(), path.canonicalize().unwrap_or(path.clone()).display());
        
        // Check if this is actually a source file change
        let abs_path = path.canonicalize().unwrap_or(path.clone());
        let abs_source_dir = source_dir.canonicalize().unwrap_or(source_dir.clone());
        let abs_theme_dir = theme_dir.canonicalize().unwrap_or(theme_dir.clone());
        let abs_config_file = config_file.canonicalize().unwrap_or(config_file.clone());
        
        let is_source_change = abs_path.starts_with(&abs_source_dir) 
            || abs_path.starts_with(&abs_theme_dir) 
            || abs_path == abs_config_file;
            
        if !is_source_change {
            println!("  Skipping non-source file change");
            continue;
        }

        // Rebuild site - the dev server will detect output changes and reload
        match build_site_with_livereload(
            &config,
            &source_dir,
            &output_dir,
            &theme_dir,
            &livereload_host,
        ) {
            Ok(_) => {
                println!("Site rebuilt successfully");
            }
            Err(e) => {
                eprintln!("Build error: {}", e);
            }
        }
    }

    Ok(())
}

/// Build site with livereload support - CLI-specific function
fn build_site_with_livereload(
    config: &crate::config::ZapConfig,
    source_dir: &Path,
    output_dir: &Path,
    theme_dir: &Path,
    livereload_host: &str,
) -> Result<()> {
    // First do the standard build
    build_site(&config.site, source_dir, output_dir, theme_dir)?;
    
    // Then add livereload script to all HTML files
    inject_livereload_into_html_files(output_dir, livereload_host)?;
    
    Ok(())
}

/// Inject livereload script into all HTML files in the output directory
fn inject_livereload_into_html_files(output_dir: &Path, livereload_host: &str) -> Result<()> {
    use std::fs;
    
    // Find all HTML files recursively
    fn process_directory(dir: &Path, livereload_host: &str) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                process_directory(&path, livereload_host)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("html") {
                // Read the HTML file
                let content = fs::read_to_string(&path)?;
                
                // Inject livereload script
                let updated_content = inject_livereload_script(&content, livereload_host);
                
                // Write back if changed
                if content != updated_content {
                    fs::write(&path, updated_content)?;
                }
            }
        }
        Ok(())
    }
    
    process_directory(output_dir, livereload_host)
}

/// Inject livereload script into HTML content
fn inject_livereload_script(html: &str, livereload_host: &str) -> String {
    let script = format!(
        r#"
   <script>
   (function() {{
       console.log('Initializing live reload...');
       const socket = new WebSocket('ws://{}/__livereload');
       
       socket.onopen = function() {{
           console.log('Live reload connected');
       }};
       
       socket.onmessage = function(event) {{
           console.log('Live reload message:', event.data);
           if (event.data === 'reload') {{
               console.log('Reloading page...');
               location.reload();
           }}
       }};
       
       socket.onclose = function() {{
           console.log('Live reload disconnected');
       }};
       
       socket.onerror = function(error) {{
           console.error('Live reload error:', error);
       }};
       
       window.addEventListener('beforeunload', function() {{
           socket.close();
       }});
   }})();
   </script>
"#,
        livereload_host
    );

    // Try to inject before closing head tag, or before body if not found
    if let Some(pos) = html.rfind("</head>") {
        let mut result = String::with_capacity(html.len() + script.len());
        result.push_str(&html[..pos]);
        result.push_str(&script);
        result.push_str(&html[pos..]);
        result
    } else if let Some(pos) = html.rfind("</body>") {
        let mut result = String::with_capacity(html.len() + script.len());
        result.push_str(&html[..pos]);
        result.push_str(&script);
        result.push_str(&html[pos..]);
        result
    } else {
        format!("{}{}", html, script)
    }
}




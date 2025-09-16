# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Zap is a static site generator written in Rust that creates beautiful project websites with minimal configuration. It processes Markdown files from a `site/` directory and generates HTML output using Tera templates.

## Architecture

The project is structured as a Rust workspace with three crates:

- **zap-cli**: Main CLI application that orchestrates the site generation process
- **zap-core**: Core library containing the site scanning and parsing logic
- **zap-render**: Render utilities (currently minimal, contains placeholder code)

### Key Components

- **Page System**: Pages are categorized by type (Home, Changelog, Index, Regular) based on filename patterns
- **Collections**: Directories containing multiple markdown files are treated as collections
- **Template Engine**: Uses Tera for HTML templating with templates in `theme/` directory
- **Navigation**: Auto-generates navigation from scanned pages and collections

## Development Commands

### Build and Run
```bash
# Build the project
cargo build

# Run the CLI
cargo run -p zap-cli

# Development mode with auto-reload and live preview
make dev
```

### Development Workflow
The `make dev` command runs:
- `cargo watch` to rebuild on Rust changes
- `tailwindcss --watch` for CSS compilation
- `live-server` for browser auto-refresh

### Theme Development
The theme uses:
- Tailwind CSS for styling
- Tera templates in `theme/` directory
- Output compiled to `out/` directory

## File Structure Patterns

- `site/README.md` → Home page (PageType::Home)
- `site/CHANGELOG.md` → Changelog page (PageType::Changelog)  
- `site/*/index.md` → Collection index pages (PageType::Index)
- `site/*.md` → Regular pages (PageType::Regular)
- `site/*/` → Collections (directories with markdown files)

## Important Implementation Details

- The CLI scans the `site/` directory for markdown files and directories
- Page titles are extracted from the first heading in each markdown file
- URL structure follows the filesystem structure with `.html` extensions
- Templates are selected based on PageType in `get_page_template()`
- Navigation is built dynamically from all discovered pages and collections
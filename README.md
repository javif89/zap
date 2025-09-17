# Zap SSG

A modern static site generator that creates beautiful project websites with (near) zero configuration.

## Features

- **Zero Configuration** - Works out of the box with sensible defaults
- **Markdown First** - Write content in Markdown, get beautiful websites
- **Syntax Highlighting** - Built-in code highlighting with multiple themes

### Basic Usage

```bash
# Build your site (default: ./site -> ./out)
zap build

# Start development server with live reload
zap serve

# Use custom directories
zap build --source ./content --output ./public --theme ./my-theme

# Start dev server on custom port
zap serve --port 8080 --open
```

## Configuration

Zap supports cascading configuration with the following priority order:

1. **CLI Arguments** (highest priority)
2. **Environment Variables** (`ZAP_*`)
3. **Configuration File** (`zap.toml`)
4. **Built-in Defaults** (lowest priority)

### Configuration File

Create a `zap.toml` file in your project root:

```toml
[build]
source = "./site"
output = "./out" 
theme = "./theme"
host = "127.0.0.1"
port = 3000
open = false

[site]
title = "My Awesome Site"
tagline = "Built with Zap"
secondary_tagline = "Fast, simple, powerful"
small_tag = "v1.0"

[home]
hero = true

[[home.features]]
title = "Lightning Fast"
description = "Blazing fast static site generation"

[[home.features]]
title = "Zero Configuration"
description = "Works out of the box with sensible defaults"

[home.primary_action]
text = "Get Started"
link = "/installation"

[home.secondary_action]
text = "View on GitHub"
link = "https://github.com/example/project"
```

### Environment Variables

All configuration options can be set via environment variables with the `ZAP_` prefix:

```bash
# Build configuration
export ZAP_BUILD__SOURCE="./content"
export ZAP_BUILD__OUTPUT="./public"
export ZAP_BUILD__PORT="8080"

# Site configuration  
export ZAP_SITE__TITLE="My Site"
export ZAP_SITE__TAGLINE="Powered by Zap"

# Run with environment config
zap build
```

### CLI Arguments

All build settings can be overridden via command line:

```bash
# Build command options
zap build \
  --source ./content \
  --output ./public \
  --theme ./custom-theme \
  --config ./my-config.toml

# Serve command options  
zap serve \
  --source ./content \
  --output ./public \
  --host 0.0.0.0 \
  --port 8080 \
  --open
```

## Content Structure

### Homepage

The `README.md` or `index.md` in your source directory becomes your homepage. Zap automatically extracts:

- **Title** - First heading (unless overridden in config)
- **Tagline** - First paragraph (unless overridden in config)

### Pages

All `.md` files in your source directory become pages:

```markdown
# Page Title

This becomes the page content.

## Section Heading

More content here.
```

### Collections

Organize related content in subdirectories:

```
site/
├── README.md
├── installation.md
└── configuration/
    ├── index.md           # Collection landing page
    ├── basic-setup.md
    └── advanced/
        └── custom-themes.md
```
# Zap SSG

A modern static site generator that creates beautiful project websites with (near) zero configuration.

This is the info from the site, not the default.

## Features

- Search and launch desktop applications
- Search files in your home directories (Documents, Pictures, Music, Videos)
- Fuzzy search that actually works
- Fast startup with background file indexing
- GTK4 with proper layer shell integration

## What is this?

Waycast is an application launcher built for Wayland desktops. It's fast, extensible, and designed to get out of your way while helping you find what you need.

## Why Another Launcher?

Linux desktop launchers are either too basic (dmenu, wofi) or too bloated (some KDE thing with 47 configuration tabs). Raycast nailed the UX on macOS, but there's no good equivalent for Linux.

Waycast aims to be:
- **Fast** - Sub-100ms search responses, instant startup
- **Clean** - Good defaults, minimal configuration needed  
- **Extensible** - Plugin system for custom functionality
- **Native** - Proper Wayland integration, not an Electron app

## Installation

### Nix Flakes

Add to your `flake.nix` inputs:
```nix
waycast.url = "git+https://gitgud.foo/thegrind/waycast";
```

Add the overlay and Home Manager module:
```nix
nixpkgs.overlays = [ inputs.waycast.overlays.default ];

home-manager.users.youruser = {
  imports = [ inputs.waycast.homeManagerModules.default ];
  
  programs.waycast = {
    enable = true;
    settings = {
      plugins.projects = {
        search_paths = ["/absolute/path/to/search"];
        skip_dirs = [ "node_modules" "target" ".git" ];
        open_command = "code -n {path}";
      };
      plugins.file_search = {
        search_paths = ["/absolute/path/to/search"];
        ignore_dirs = ["scripts", "temp"];
      };
    };
    css = ''
      window {
        background: rgba(0, 0, 0, 0.8);
        border-radius: 12px;
      }
    '';
  };
};
```

**Just the package:**
```nix
nixpkgs.overlays = [ inputs.waycast.overlays.default ];
environment.systemPackages = [ pkgs.waycast ];
# or for home-manager:
home.packages = [ pkgs.waycast ];
```

## Contributing

TBA

## License

TBA
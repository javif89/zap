# Configuration Files

You can configure waycast through editing:

- waycast.toml
- waycast.css

Both of these can be found in your config directory (usually `~/.config/waycast`).

If you're on Nix or use Nix Home Manager, we also provide a [home manager module](/configuration/nix-home-manager).

## Example Configurations

### waycast.toml


### waycast.css

Take a look at the [CSS Example](/configuration/css-configuration) for the available elements. Waycast will follow
your system's GTK theme by default, but you can override this by creating a `waycast.css` in your config directory or
through the [home manager module](/configuration/nix-home-manager).

## Plugin Default Values

Plugins have certain default values that get merged in with your configuration to save you the time
of including things you most likely want.

### File Search

**search_paths:**

- ~/Documents
- ~/Pictures 
- ~/Videos 
- ~/Music 
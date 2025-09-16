# Home Manager Module

If you installed waycast through nix, and use [Nix Home Manager](https://github.com/nix-community/home-manager), you can manage
your configuration declaratively.

The home manager module doesn't have any special syntax. Everything under the `settings` key just gets translated to toml. So
you should be able to directly translate any of the configuration examples relatively easily.

## Example

```nix
{
  pkgs,
  config,
  inputs,
  ...
}:

{
  imports = [ inputs.waycast.homeManagerModules.default ];
  programs.waycast = {
    enable = true;
    settings = {
      plugins.file_search = {
        search_paths = [
          "/home/youruser/your-folder"
        ];
      };
      plugins.projects = {
        open_command = "code -n {path}";
        search_paths = [
          "/home/youruser/projects"
        ];
      };
    };
  };
}

```
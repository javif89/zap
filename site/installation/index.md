
# Installation

## Dependencies

WayCast requires the following packages to work:

- gtk4
- librsvg
- gio
- gtk4-layer-shell
- wayland

If you follow the installation steps below, all of this will be taken care of for you.

## Nix Flakes

Add to your `flake.nix` inputs:
```nix
waycast.url = "git+https://gitgud.foo/thegrind/waycast";
```

Add the overlay:

```nix
  overlays = [
    inputs.waycast.overlays.default
  ];
```

We also provide a [home manager module](/configuration/nix-home-manager).

**Just the package:**
```nix
nixpkgs.overlays = [ inputs.waycast.overlays.default ];
environment.systemPackages = [ pkgs.waycast ];
# or for home-manager:
home.packages = [ pkgs.waycast ];
```

## Nix Build

If you don't use Nix to manage your packages, you can still use it to build from source.

Just clone the repo and run:

```sh
nix build .
```

## Build from source

You'll need the rust toolchain as long as the dependencies outlined above. Then you can just run:

```sh
cargo install
```
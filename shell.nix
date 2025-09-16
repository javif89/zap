{
  pkgs ? import <nixpkgs> { },
}:

pkgs.mkShell {
  buildInputs = with pkgs; [
    cargo-watch
    tailwindcss_4
    concurrently
  ];
}

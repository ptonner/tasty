{
  description = "Reproducible macroquad development";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        lib = pkgs.lib;
        inherit (pkgs) rust-bin makeRustPlatform;
        rustPlatform = makeRustPlatform {
          cargo = rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
          rustc = rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
        };
        nativeBuildInputs = with pkgs; [
          pkg-config
          (rust-bin.stable.latest.default.override { targets = [ "wasm32-unknown-unknown" ]; })
          wasm-bindgen-cli
        ];
        buildInputs = with pkgs; [
          libGL
          udev
          alsa-lib
          vulkan-loader
          mesa
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr # To use the x11 feature
          libxkbcommon
          wayland # To use the wayland feature
        ];
        webBuildInputs = builtins.concatLists [
          (with pkgs; [
            simple-http-server
            wasm-bindgen-cli
            trunk
          ])
          buildInputs
        ];
      in
      {
        devShells.default = pkgs.mkShell rec {
          inherit nativeBuildInputs;
          buildInputs = webBuildInputs;
          LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
        };
      }
    );
}

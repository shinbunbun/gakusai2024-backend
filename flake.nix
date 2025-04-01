{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
      in
      {
        devShell = pkgs.mkShell {
          name = "gakusai2024-backend";
          buildInputs = [
            pkgs.sea-orm-cli
            pkgs.protobuf
          ];
          packages = [
            (pkgs.rust-bin.stable."1.84.0".default.override {
              extensions = [ "rust-src" "rustc" "cargo" "rustfmt" "clippy" ];
            })
          ];
        };
      }
    );
}

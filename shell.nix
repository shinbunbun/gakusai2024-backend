{ pkgs ? import <nixpkgs> {} }:

let
  sea-orm-cli = pkgs.rustPlatform.buildRustPackage rec {
    pname = "sea-orm-cli";
    version = "1.0.1";

    src = pkgs.fetchCrate {
      inherit pname version;
      hash = "sha256-b1Nlt3vsLDajTiIW9Vn51Tv9gXja8/ZZBD62iZjh3KY=";
    };

    nativeBuildInputs = [ pkgs.pkg-config ];

    buildInputs = [ pkgs.openssl ]
      ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.SystemConfiguration ];

    cargoHash = "sha256-ZGM+Y67ycBiukgEBUq+WiA1OUCGahya591gM6CGwzMQ=";
  };
in
pkgs.mkShell {
  buildInputs = [
    pkgs.rustup
    pkgs.protobuf
    pkgs.libiconv
    pkgs.evans
    pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
    sea-orm-cli
  ];
}

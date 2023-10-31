{
  description = "gameboy emulator";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    naersk.url = "github:nmattia/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, naersk, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };
        naersk' = pkgs.callPackage naersk { };
      in
      rec
      {
        defaultPackage = naersk'.buildPackage
          {
            src = ./.;
            override = p: {
              buildInputs = with pkgs; p.buildInputs ++ [
                SDL2
                pkg-config
              ] ++ lib.optionals stdenv.isDarwin [
                darwin.apple_sdk.frameworks.Security
              ];
            };
          };

        devShell = pkgs.mkShell
          {
            nativeBuildInputs = with pkgs; [
              rustc
              rustfmt
              rust-analyzer
              cargo
            ] ++ defaultPackage.buildInputs;

            RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          };
      }
    );
}

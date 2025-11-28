{
  description = "Rust devshell with pandoc";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        typst2latex = pkgs.rustPlatform.buildRustPackage {
          pname = "typst2latex";
          version = "0.1.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.pandoc ];
        };

      in
      {
        devShells.default = with pkgs; mkShell {
          buildInputs = [
            pkg-config
            pandoc
            rust-analyzer
            rust-bin.beta.latest.default
          ];
        };

        packages = { default = typst2latex; };

      }
    );
}

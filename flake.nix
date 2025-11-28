{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { rust-overlay, flake-utils, nixpkgs, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
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

        packages = {
          default =
            pkgs.rustPlatform.buildRustPackage {
              pname = "typst2latex";
              version = "0.1.0";
              src = ./.;
              cargoLock = {
                lockFile = ./Cargo.lock;
                allowBuiltinFetchGit = true;
              };
              meta = with pkgs.lib; {
                description = "An oppiniated converter from unequivocal-ams typst documents to latex.";
                homepage = "https://github.com/aualbert/typst2latex";
                license = licenses.mit;
                maintainers = [ ];
              };
            };
        };
      }
    );
}

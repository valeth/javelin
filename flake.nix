{
    description = "Javelin development environment";

    inputs = {
        nixpkgs.url      = "github:nixos/nixpkgs/nixpkgs-unstable";
        flake-utils.url  = "github:numtide/flake-utils";
        rust-overlay.url = "github:oxalica/rust-overlay";
    };

    outputs = { nixpkgs, flake-utils, rust-overlay, ... }:
        flake-utils.lib.eachDefaultSystem (system:
            let
                overlays = [ (import rust-overlay) ];
                pkgs = import nixpkgs { inherit system overlays; };

                rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
                rustNightlyToolchain = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.minimal.override {
                    extensions = [ "rustfmt" ];
                    targets = [ "x86_64-unknown-linux-gnu" ];
                });

                buildTools = [
                    rustToolchain
                    rustNightlyToolchain
                ];
            in {
                devShells.default = pkgs.mkShell {
                    name = "javelin";

                    nativeBuildInputs = buildTools;

                    packages = with pkgs; [
                        cargo-deny
                        sqlx-cli
                    ];
                };
            });
}

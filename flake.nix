{
    description = "Javelin development shell";

    inputs = {
        nixpkgs.url      = "github:nixos/nixpkgs/nixpkgs-unstable";
        flake-utils.url  = "github:numtide/flake-utils";
        rust-overlay.url = "github:oxalica/rust-overlay";
    };

    outputs = { nixpkgs, flake-utils, rust-overlay, ... }:
        flake-utils.lib.eachDefaultSystem (system:
            let
                lib = nixpkgs.lib;
                overlays = [ (import rust-overlay) ];
                pkgs = import nixpkgs { inherit system overlays; };
                toolchains = [
                    (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
                    (lib.hiPrio pkgs.rust-bin.nightly."2024-08-01".rustfmt)
                ];
            in
            with pkgs; {
                devShells.default = mkShell {
                    name = "javelin";

                    nativeBuildInputs = [
                        pkg-config
                    ];

                    packages = [
                        cargo-deny
                        sqlx-cli
                    ] ++ toolchains;
                };
            });
}

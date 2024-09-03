{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-parts.url = "github:hercules-ci/flake-parts";
    cargo-watchdoc.url = "github:ModProg/cargo-watchdoc";
  };

  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; }
      {
        systems = [
          "x86_64-linux"
          "aarch64-linux"
        ];

        perSystem = { self', lib, system, pkgs, config, ... }: {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;

            overlays = with inputs; [
              rust-overlay.overlays.default
            ];
          };

          apps.default = {
            type = "app";
            program = self'.packages.default;
          };

          packages.default = pkgs.callPackage (import ./nix/package.nix) { };

          devShells.default =
            let
              rust-toolchain = (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml).override {
                extensions = [ "rust-src" "rust-analyzer" ];
              };
            in
            pkgs.mkShell {
              packages = with pkgs; [
                pkg-config
                openssl
              ] ++ [ rust-toolchain inputs.cargo-watchdoc.packages.${system}.default ];
            };
        };
      };
}

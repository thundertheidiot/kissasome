{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      lib = nixpkgs.lib;

      forAllSystems =
        f:
        lib.genAttrs lib.systems.flakeExposed (
          system:
          f (
            import nixpkgs {
              inherit system;
              overlays = [ (import rust-overlay) ];
            }
          )
        );

      rustToolchain =
        pkgs:
        pkgs.rust-bin.nightly.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "clippy"
            "rustfmt"
          ];
        };
    in
    {
      packages = forAllSystems (pkgs: { });

      devShell = forAllSystems (
        pkgs:
        pkgs.mkShell {
          packages = with pkgs; [
            gnumake

            # backend
            (rustToolchain pkgs)

            sea-orm-cli

            # sqlite
            pkg-config
            sqlite
            gcc

            # frontend
            nodejs_26
          ];
        }
      );
    };
}

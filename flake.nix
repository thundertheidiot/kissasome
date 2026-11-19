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
      packages = forAllSystems (
        pkgs:
        let
          rustPlatform = pkgs.makeRustPlatform {
            cargo = rustToolchain pkgs;
            rustc = rustToolchain pkgs;
          };

          src = ./.;

          dist = pkgs.buildNpmPackage {
            pname = "kissasome-dist";
            version = "0.1.0";
            inherit src;
            nodejs = pkgs.nodejs_26;
            npmDepsHash = "sha256-Mhju4oo1EpAHCFwk3fY7dGIe3GfQKRyP9YmtEMtvNZc=";
            installPhase = "cp -r dist $out";
          };
        in
        {
          inherit dist;

          default = rustPlatform.buildRustPackage {
            pname = "kissasome";
            version = "0.1.0";
            inherit src;
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = with pkgs; [
              pkg-config
              makeWrapper
            ];
            buildInputs = [ pkgs.sqlite ];
            doCheck = false;
            postInstall = ''
              mkdir -p $out/
              ln -s ${dist} $out/dist
              wrapProgram $out/bin/kissasome --set-default DIST_PATH $out/dist
            '';
            meta.mainProgram = "kissasome";
          };
        }
      );

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

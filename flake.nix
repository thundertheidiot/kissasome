{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  };

  outputs =
    { nixpkgs, ... }:
    let
      lib = nixpkgs.lib;

      forAllSystems =
        f: lib.genAttrs lib.systems.flakeExposed (system: f nixpkgs.legacyPackages."${system}");
    in
    {
      packages = forAllSystems (pkgs: { });

      devShell = forAllSystems (
        pkgs:
        pkgs.mkShell {
          packages = with pkgs; [
            sea-orm-cli
          ];
        }
      );
    };
}

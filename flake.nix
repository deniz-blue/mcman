{
  description = "Powerful Minecraft Server Manager CLI";
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
  };

  outputs = {
    nixpkgs,
    crane,
    ...
  }:
    let
      systems = nixpkgs.legacyPackages.x86_64-linux.rustc.meta.platforms;

      mkMcman = pkgs:
        let
          craneLib = crane.mkLib pkgs;

          src = pkgs.lib.cleanSourceWith {
            src = ./.;
            filter = path: type:
              (craneLib.filterCargoSources path type)
              || (builtins.match ".*res/.*" path != null);
            name = "mcman";
          };

          common = {
            inherit src;
            strictDeps = true;
            doCheck = false;
          };

          cargoArtifacts = craneLib.buildDepsOnly common;

        in craneLib.buildPackage (common // {inherit cargoArtifacts;});

    in {
      packages = nixpkgs.lib.genAttrs systems
        (system: rec {
          mcman = mkMcman nixpkgs.legacyPackages.${system};
          default = mcman;
        });

      overlays.default = self: super: {
        mcman = mkMcman self;
      };
    };
}

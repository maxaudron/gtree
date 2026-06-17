{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-26.05";
    parts.url = "github:hercules-ci/flake-parts";
    parts.inputs.nixpkgs-lib.follows = "nixpkgs";
    crane.url = "github:ipetkov/crane";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{
      self,
      parts,
      crane,
      fenix,
      ...
    }:
    parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      perSystem =
        {
          self',
          pkgs,
          config,
          lib,
          system,
          ...
        }:
        let
          toolchain = fenix.packages.${system}.stable;
          craneLib = (crane.mkLib pkgs).overrideToolchain toolchain.toolchain;

          # Common arguments can be set here to avoid repeating them later
          # Note: changes here will rebuild all dependency crates
          commonArgs = {
            src = craneLib.cleanCargoSource ./.;
            strictDeps = true;

            nativeBuildInputs = with pkgs; [
              pkg-config
              autoPatchelfHook
            ];

            buildInputs = with pkgs; [
              libgit2
              openssl
              aws-lc
              file
            ];
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          fileSetForCrate = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              ./Cargo.toml
              ./Cargo.lock
              (craneLib.fileset.commonCargoSources ./src)
              ./graphql
            ];
          };

          gtree = craneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;
              src = fileSetForCrate;
            }
          );
        in
        {
          devShells.default = craneLib.devShell {
            checks = self.checks;

            inputsFrom = [ gtree ];
            packages = [ toolchain.rust-analyzer ];
          };

          packages = {
            default = gtree;
          };
        };
    };
}

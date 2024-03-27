{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    nci.url = "github:yusdacra/nix-cargo-integration";
    nci.inputs.nixpkgs.follows = "nixpkgs";
    parts.url = "github:hercules-ci/flake-parts";
    parts.inputs.nixpkgs-lib.follows = "nixpkgs";
  };

  outputs = inputs@{ parts, nci, ... }:
    parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "x86_64-darwin" "aarch64-linux" "aarch64-darwin" ];
      imports = [ nci.flakeModule ];
      perSystem = { pkgs, config, lib, ... }:
        let
          # shorthand for accessing this crate's outputs
          # you can access crate outputs under `config.nci.outputs.<crate name>` (see documentation)
          crateOutputs = config.nci.outputs."gtree";
        in
        {
          nci = {
            projects."gtree".path = ./.;
            crates.gtree =
              let mkDerivation = {
                # inputs and most other stuff will automatically merge
                buildInputs = lib.optional pkgs.stdenv.isDarwin [
                  pkgs.darwin.libiconv
                  pkgs.darwin.apple_sdk.frameworks.Security
                  pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
                ];
              };
              in
              {
                drvConfig = { inherit mkDerivation; };
                depsDrvConfig = { inherit mkDerivation; };
              };

            toolchainConfig = {
              channel = "stable";
              components = [ "rustfmt" "rust-src" ];
            };
          };


          devShells.default = crateOutputs.devShell;
          packages.default = crateOutputs.packages.release;
        };
    };
}

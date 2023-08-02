{
  nixConfig = {
    substituters =
      [ "https://cache.nixos.org/" "https://nix-community.cachix.org" ];
    trusted-public-keys = [
      "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY="
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
    ];
  };
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-23.05";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };

    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, fenix, crane, advisory-db }:
    with nixpkgs.lib;
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        rustToolchain = with fenix.packages.${system};
          combine ([ latest.toolchain ] ++ (if system == "x86_64-linux" then
            [ targets.x86_64-unknown-linux-musl.latest.rust-std ]
          else
            [ ]));

        openssl =
          (if (system == "aarch64-darwin" || system == "x86_64-darwin") then
            pkgs.openssl.override { static = true; }
          else
            pkgs.pkgsStatic.openssl);

        nativeBuildInputs = with pkgs;
          [
            rustToolchain
            pkg-config

            openssl
          ] ++ lib.optional stdenv.isDarwin [
            pkgs.darwin.libiconv
            pkgs.darwin.apple_sdk.frameworks.Security
          ];

        OPENSSL_STATIC = "true";
        OPENSSL_DIR = "${openssl}";
        OPENSSL_LIB_DIR = "${openssl.out}/lib";
        OPENSSL_CRYPTO_LIBRARY = "${openssl.out}/lib";
        OPENSSL_INCLUDE_DIR = "${openssl.dev}/include";
        CARGO_BUILD_TARGET = if system == "x86_64-linux" then
          "x86_64-unknown-linux-musl"
        else
          null;
        CARGO_BUILD_RUSTFLAGS = if system == "x86_64-linux" then
          "-C target-feature=+crt-static"
        else
          null;

        graphqlFilter = path: _type: builtins.match ".*graphql$" path != null;
        markdownFilter = path: _type: builtins.match ".*md$" path != null;
        markdownOrCargo = path: type:
          (graphqlFilter path type) || (markdownFilter path type)
          || (craneLib.filterCargoSources path type);

        # crane setup
        craneLib = crane.lib.${system}.overrideToolchain rustToolchain;
        src = nixpkgs.lib.cleanSourceWith {
          src = craneLib.path ./.; # The original, unfiltered source
          filter = markdownOrCargo;
        };

        cargoArtifacts = self.packages.${system}.cargoArtifacts;
      in {
        packages = {
          cargoArtifacts = craneLib.buildDepsOnly {
            inherit src;
            inherit nativeBuildInputs OPENSSL_STATIC OPENSSL_DIR OPENSSL_LIB_DIR
              OPENSSL_CRYPTO_LIBRARY OPENSSL_INCLUDE_DIR CARGO_BUILD_TARGET
              CARGO_BUILD_RUSTFLAGS;
          };

          #####################################
          # Build Binaries
          gtree = craneLib.buildPackage {
            inherit cargoArtifacts src;
            inherit nativeBuildInputs OPENSSL_STATIC OPENSSL_DIR OPENSSL_LIB_DIR
              OPENSSL_CRYPTO_LIBRARY OPENSSL_INCLUDE_DIR CARGO_BUILD_TARGET
              CARGO_BUILD_RUSTFLAGS;
          };

          default = self.packages.${system}.gtree;
        };

        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs OPENSSL_STATIC OPENSSL_DIR OPENSSL_LIB_DIR
            OPENSSL_CRYPTO_LIBRARY OPENSSL_INCLUDE_DIR CARGO_BUILD_TARGET
            CARGO_BUILD_RUSTFLAGS;
        };

        formatter = pkgs.nixfmt;
      });
}

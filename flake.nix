{
  nixConfig = {
    substituters =
      [ "https://cache.nixos.org/" "https://nix-community.cachix.org" "https://nix.cache.vapor.systems" ];
    trusted-public-keys = [
      "cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY="
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
      "nix.cache.vapor.systems-1:OjV+eZuOK+im1n8tuwHdT+9hkQVoJORdX96FvWcMABk="
    ];
  };
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk = {
      url = "github:nmattia/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, utils, naersk }:
    with nixpkgs.lib;
    recursiveUpdate (utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        rust = rust-overlay.packages."${system}".rust;

        naersk-lib = naersk.lib."${system}".override {
          cargo = rust;
          rustc = rust;
        };
      in rec {
        packages.default = packages.gtree;

        packages.gtree = naersk-lib.buildPackage {
          pname = "gtree";
          root = ./.;

          nativeBuildInputs = [ pkgs.perl ];

          cargoBuildOptions = prev:
            prev ++ [ "--features=vendored-libgit2,vendored-openssl" ];
        };

        apps.default = utils.lib.mkApp { drv = packages.default; };

        devShell = pkgs.mkShell { nativeBuildInputs = [ rust pkgs.perl ]; };
      })) (utils.lib.eachSystem [
        utils.lib.system.x86_64-linux
        utils.lib.system.aarch64-linux
      ] (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};

          staticInputs = with pkgs; [
            pkgsStatic.stdenv.cc
            pkgsStatic.openssl.dev
            rust
            perl
          ];

          target =
            "${elemAt (strings.splitString "-" system) 0}-unknown-linux-musl";

          rust = rust-overlay.packages."${system}".rust.override {
            extensions = [ "rust-src" ];
            targets = [ target ];
          };

          naersk-lib = naersk.lib."${system}".override {
            cargo = rust;
            rustc = rust;
          };

          gitlab-upload = makeBinScript "gitlab-upload" ''
            ${pkgs.curl} -f --header "PRIVATE-TOKEN: $GITLAB_API_TOKEN" --upload-file result/bin/gtree https://gitlab.com/api/v4/projects/${project_id}/packages/generic/${name}/${version}/$1
          '';
        in rec {
          # `nix build`
          packages.gtreeStatic = naersk-lib.buildPackage {
            pname = "gtree";
            root = ./.;

            nativeBuildInputs = staticInputs;

            CARGO_BUILD_TARGET = target;
            CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS =
              "-C target-feature=+crt-static";
            CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS =
              "-C target-feature=+crt-static";

            OPENSSL_STATIC = true;
            OPENSSL_DIR = "${pkgs.pkgsStatic.openssl}";
            OPENSSL_LIB_DIR = "${pkgs.pkgsStatic.openssl.out}/lib";
            OPENSSL_INCLUDE_DIR = "${pkgs.pkgsStatic.openssl.dev}/include";
          };

          devShell = pkgs.mkShell {
            nativeBuildInputs = staticInputs;

            shellHook = ''
              export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-C target-feature=+crt-static"
              export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_RUSTFLAGS="-C target-feature=+crt-static"

              export OPENSSL_STATIC=1
              export OPENSSL_DIR="${pkgs.pkgsStatic.openssl}"
              export OPENSSL_LIB_DIR="${pkgs.pkgsStatic.openssl.out}/lib"
              export OPENSSL_INCLUDE_DIR="${pkgs.pkgsStatic.openssl.dev}/include"
            '';
          };
        }));
}

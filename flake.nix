{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    gitignore = {
      url = "github:hercules-ci/gitignore.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, utils, gitignore, flake-compat }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config = { allowUnfree = true; };
        };
        inherit (gitignore.lib) gitignoreSource;

        # Sunscreen LLVM compiler for parasol target
        sunscreen-llvm = pkgs.stdenv.mkDerivation rec {
          pname = "sunscreen-llvm";
          version = "2025.09.30";
          # Asset filenames use dashes instead of dots in the date
          fileVersion = "2025-09-30";

          src =
            let
              urlBase = "https://github.com/Sunscreen-tech/sunscreen-llvm/releases/download/v${version}";
            in
            if pkgs.stdenv.isDarwin then
              pkgs.fetchurl {
                url = "${urlBase}/parasol-compiler-macos-aarch64-${fileVersion}.tar.gz";
                sha256 = "0ra93mji3j9km7ia21gsqswn49a3abwc1ml1xq643hzq4xigyqjd";
              }
            else if pkgs.stdenv.isAarch64 then
              pkgs.fetchurl {
                url = "${urlBase}/parasol-compiler-linux-aarch64-${fileVersion}.tar.gz";
                sha256 = "197fybbjvimnyqwwn3q7s9yrljbqp57s42n9znpckmnbcbp8p373";
              }
            else
              pkgs.fetchurl {
                url = "${urlBase}/parasol-compiler-linux-x86-64-${fileVersion}.tar.gz";
                sha256 = "1p0418nqzs6a2smrbqiyrxj34pimm6qzj7k29l4ys226cz6kfz2r";
              };

          nativeBuildInputs = pkgs.lib.optionals pkgs.stdenv.isLinux [
            pkgs.autoPatchelfHook
          ];

          buildInputs = pkgs.lib.optionals pkgs.stdenv.isLinux [
            pkgs.stdenv.cc.cc.lib  # Provides libstdc++ and libgcc_s
            pkgs.zlib
          ];

          sourceRoot = ".";

          unpackPhase = ''
            tar -xzf $src
          '';

          installPhase = ''
            mkdir -p $out
            cp -r * $out/
          '';

          meta = with pkgs.lib; {
            description = "Sunscreen LLVM compiler for parasol target (FHE compilation)";
            homepage = "https://github.com/Sunscreen-tech/sunscreen-llvm";
            license = licenses.agpl3Only;
            platforms = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
          };
        };

        # Build the mdbook-check-code preprocessor
        mdbook-check-code = pkgs.rustPlatform.buildRustPackage {
          pname = "mdbook-check-code";
          version = "0.1.0";
          src = gitignoreSource ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = [ pkgs.pkg-config ];

          nativeCheckInputs = [
            pkgs.rustfmt
            pkgs.clippy
            pkgs.mdbook
            sunscreen-llvm
            pkgs.nodejs
            pkgs.nodePackages.typescript
          ];

          # Override checkPhase to run fmt, clippy, tests, and fixture validation
          checkPhase = ''
            runHook preCheck

            echo "Running cargo fmt check..."
            cargo fmt --check

            echo "Running clippy on workspace only (not dependencies)..."
            cargo clippy --all-targets --workspace -- -D warnings

            echo "Running tests..."
            cargo test --release

            echo "Testing mdbook fixtures with built preprocessor..."
            export CLANG="${sunscreen-llvm}/bin/clang"
            export PATH="$PWD/target/release:$PATH"
            cd tests/fixtures
            mdbook build
            cd ../..

            runHook postCheck
          '';

          meta = with pkgs.lib; {
            description = "mdBook preprocessor for checking code blocks in multiple languages";
            homepage = "https://github.com/Sunscreen-tech/mdbook-check-code";
            license = licenses.agpl3Only;
            mainProgram = "mdbook-check-code";
          };
        };
      in {

        packages = {
          inherit mdbook-check-code sunscreen-llvm;
          default = mdbook-check-code;
        };

        apps.default = {
          type = "app";
          program = "${mdbook-check-code}/bin/mdbook-check-code";
        };

        devShells.default = with pkgs;
          mkShellNoCC {
            nativeBuildInputs = [
              # Rust toolchain
              cargo
              rustc
              rustfmt
              clippy

              # FHE compiler
              sunscreen-llvm

              # mdbook tools
              mdbook

              # TypeScript support
              nodejs
              nodePackages.typescript
            ];

            shellHook = ''
              export CLANG="${sunscreen-llvm}/bin/clang"

              echo "Development environment loaded."
              echo "Available tools:"
              echo "  cargo                - Build with 'cargo build'"
              echo "  clang (parasol)      - ${sunscreen-llvm}/bin/clang"
              echo "  node                 - Node.js runtime"
              echo "  tsc                  - TypeScript compiler"
              echo ""
              echo "Environment variables:"
              echo "  CLANG=${sunscreen-llvm}/bin/clang"
            '';
          };
      });
}

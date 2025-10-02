{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    gitignore = {
      url = "github:hercules-ci/gitignore.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, utils, gitignore }:
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

        # Build the mdbook-check-parasol preprocessor
        mdbook-check-parasol = pkgs.rustPlatform.buildRustPackage {
          pname = "mdbook-check-parasol";
          version = "0.1.0";
          src = gitignoreSource ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = [ pkgs.pkg-config ];

          meta = with pkgs.lib; {
            description = "mdBook preprocessor for checking Parasol C code blocks";
            homepage = "https://github.com/Sunscreen-tech/mdbook-check-parasol";
            license = licenses.mit;
          };
        };
      in {

        packages = {
          inherit mdbook-check-parasol sunscreen-llvm;
          default = mdbook-check-parasol;
        };

        checks = {
          # Build check
          build = mdbook-check-parasol;
        };

        devShell = with pkgs;
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

              # The preprocessor itself
              mdbook-check-parasol
            ];

            shellHook = ''
              export CLANG="${sunscreen-llvm}/bin/clang"

              echo "Development environment loaded."
              echo "Available tools:"
              echo "  mdbook-check-parasol - The preprocessor binary"
              echo "  clang (parasol)      - ${sunscreen-llvm}/bin/clang"
              echo ""
              echo "Environment variables:"
              echo "  CLANG=${sunscreen-llvm}/bin/clang"
            '';
          };
      });
}

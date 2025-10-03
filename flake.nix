{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    crane = {
      url = "github:ipetkov/crane";
      # inputs.nixpkgs.follows = "nixpkgs";
    };
    gitignore = {
      url = "github:hercules-ci/gitignore.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, utils, crane, gitignore }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config = { allowUnfree = true; };
        };
        craneLib = crane.mkLib pkgs;
        inherit (gitignore.lib) gitignoreSource;

        # Sunscreen LLVM compiler for parasol target
        sunscreen-llvm = pkgs.stdenv.mkDerivation rec {
          pname = "sunscreen-llvm";
          version = "2025.09.30";
          # Asset filenames use dashes instead of dots in the date
          fileVersion = "2025-09-30";

          src = let
            urlBase =
              "https://github.com/Sunscreen-tech/sunscreen-llvm/releases/download/v${version}";
          in if pkgs.stdenv.isDarwin then
            pkgs.fetchurl {
              url =
                "${urlBase}/parasol-compiler-macos-aarch64-${fileVersion}.tar.gz";
              sha256 = "0ra93mji3j9km7ia21gsqswn49a3abwc1ml1xq643hzq4xigyqjd";
            }
          else if pkgs.stdenv.isAarch64 then
            pkgs.fetchurl {
              url =
                "${urlBase}/parasol-compiler-linux-aarch64-${fileVersion}.tar.gz";
              sha256 = "197fybbjvimnyqwwn3q7s9yrljbqp57s42n9znpckmnbcbp8p373";
            }
          else
            pkgs.fetchurl {
              url =
                "${urlBase}/parasol-compiler-linux-x86-64-${fileVersion}.tar.gz";
              sha256 = "1p0418nqzs6a2smrbqiyrxj34pimm6qzj7k29l4ys226cz6kfz2r";
            };

          nativeBuildInputs =
            pkgs.lib.optionals pkgs.stdenv.isLinux [ pkgs.autoPatchelfHook ];

          buildInputs = pkgs.lib.optionals pkgs.stdenv.isLinux [
            pkgs.stdenv.cc.cc.lib # Provides libstdc++ and libgcc_s
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
            description =
              "Sunscreen LLVM compiler for parasol target (FHE compilation)";
            homepage = "https://github.com/Sunscreen-tech/sunscreen-llvm";
            license = licenses.agpl3Only;
            platforms = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" ];
          };
        };

        fixture-src = gitignoreSource ./tests/fixtures;
        src = craneLib.cleanCargoSource ./.;
        commonArgs = {
          inherit src;
          strictDeps = true;
        };

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
        mdbook-check-code =
          craneLib.buildPackage (commonArgs // { inherit cargoArtifacts; });

      in {
        packages = {
          inherit mdbook-check-code;
          default = mdbook-check-code;
        };

        apps.default = {
          type = "app";
          program = "${mdbook-check-code}/bin/mdbook-check-code";
        };

        checks = {
          inherit mdbook-check-code;

          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          mdbook-check-code-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          # Check formatting
          mdbook-check-code-fmt = craneLib.cargoFmt { inherit src; };

          # The script inlined for brevity, consider extracting it
          # so that it becomes independent of nix
          runE2ETests = pkgs.runCommand "e2e-tests" {
            nativeBuildInputs = with pkgs; [
              mdbook

              # C compilers
              sunscreen-llvm
              gcc

              # TypeScript compiler
              nodejs
              nodePackages.typescript
            ];
          } ''
            cp -r ${fixture-src}/* $TMPDIR/

            # Make everything in this directory writable, otherwise all the
            # commands below will fail.
            chmod -R u+w .

            export CLANG="${sunscreen-llvm}/bin/clang"
            export RUST_LOG=info

            # Replace the mdbook-check-code path in book.toml
            # to point to the built binary in this derivation.
            sed -i "s|../../target/release/mdbook-check-code|${mdbook-check-code}/bin/mdbook-check-code|g" book.toml

            mdbook build

            # After the build is successful, copy the final output to the expected $out path.
            mkdir $out
            cp -r $TMPDIR/tests/fixtures/book/* $out
          '';
        };

        devShells.default = with pkgs;
          craneLib.devShell {
            nativeBuildInputs = [
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

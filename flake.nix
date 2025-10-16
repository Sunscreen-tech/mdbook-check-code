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
    sunscreen-llvm = {
      url = "github:Sunscreen-tech/sunscreen-llvm/sunscreen";
    };
  };

  outputs = { self, nixpkgs, utils, crane, gitignore, sunscreen-llvm }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config = { allowUnfree = true; };
        };
        craneLib = crane.mkLib pkgs;
        inherit (gitignore.lib) gitignoreSource;

        # Sunscreen LLVM compiler for parasol target (from flake input)
        sunscreen-llvm-pkg = sunscreen-llvm.packages.${system}.default;

        # Build mdbook-check-code using package.nix
        mdbook-check-code =
          pkgs.callPackage ./package.nix { inherit craneLib gitignoreSource; };

        # For checks: need to recreate src, commonArgs, and cargoArtifacts
        src = craneLib.cleanCargoSource ./.;
        commonArgs = {
          inherit src;
          strictDeps = true;
        };
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        fixture-src = gitignoreSource ./tests/fixtures;

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
              sunscreen-llvm-pkg
              gcc

              # Language compilers
              nodejs
              nodePackages.typescript
              solc
            ];
          } ''
            cp -r ${fixture-src}/* $TMPDIR/

            # Make everything in this directory writable, otherwise all the
            # commands below will fail.
            chmod -R u+w .

            export CLANG="${sunscreen-llvm-pkg}/bin/clang"
            export RUST_LOG=info

            # Set XDG_DATA_HOME to a temporary location for approval storage
            # This allows the approval mechanism to work in the nix sandbox
            export XDG_DATA_HOME=$TMPDIR/xdg-data

            # Replace the mdbook-check-code path in book.toml
            # to point to the built binary in this derivation.
            sed -i "s|../../target/release/mdbook-check-code|${mdbook-check-code}/bin/mdbook-check-code|g" book.toml

            # Approve the book.toml for security
            ${mdbook-check-code}/bin/mdbook-check-code allow

            mdbook build

            # After the build is successful, copy the final output to the expected $out path.
            mkdir $out
            cp -r $TMPDIR/book/* $out
          '';
        };

        devShells.default = with pkgs;
          craneLib.devShell {
            buildInputs = [
              # C compilers
              sunscreen-llvm-pkg
              gcc

              # mdbook tools
              mdbook

              # Language compilers
              nodejs
              nodePackages.typescript
              solc
            ];

            shellHook = ''
              export CLANG="${sunscreen-llvm-pkg}/bin/clang"

              echo "Development environment loaded."
              echo "Available tools:"
              echo "  cargo                - Build with 'cargo build'"
              echo "  clang (parasol)      - ${sunscreen-llvm-pkg}/bin/clang"
              echo "  gcc                  - C compiler"
              echo "  node                 - Node.js runtime"
              echo "  tsc                  - TypeScript compiler"
              echo "  solc                 - Solidity compiler"
              echo ""
              echo "Environment variables:"
              echo "  CLANG=${sunscreen-llvm-pkg}/bin/clang"
            '';
          };
      });
}

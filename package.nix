{ craneLib, gitignoreSource }:

let
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
in craneLib.buildPackage (commonArgs // { inherit cargoArtifacts; })

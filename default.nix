{ pkgs }:

let
  # Read the flake.lock to get locked dependency versions
  lock = builtins.fromJSON (builtins.readFile ./flake.lock);

  # Fetch crane from the locked version
  craneSrc = builtins.fetchTree lock.nodes.crane.locked;
  craneLib = import craneSrc { inherit pkgs; };

  # Fetch gitignore.nix from the locked version
  gitignoreSrc = builtins.fetchTree lock.nodes.gitignore.locked;
  gitignoreLib = import gitignoreSrc { inherit (pkgs) lib; };
  gitignoreSource = gitignoreLib.gitignoreSource;

  # Import package.nix directly
  package = import ./package.nix { inherit craneLib gitignoreSource; };
in
package

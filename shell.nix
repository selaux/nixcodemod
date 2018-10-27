{ pkgs ? import ./nix/nixpkgs.nix }:

pkgs.mkShell {
  buildInputs = [ pkgs.latest.rustChannels.nightly.rust ] ++ pkgs.stdenv.lib.optionals pkgs.stdenv.isDarwin [ pkgs.darwin.apple_sdk.frameworks.Security ];
}

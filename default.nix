{ pkgs ? import ./nixpkgs.nix {} }:

with pkgs;

let
    #
    # Make holo-nixpkg's rustPlatform and cargo available here.  This allows us
    # to make cargo/rustc available to build DNAs in a functionally defined way.
    # Once we have reliable builds running on Hydra, this will make eg. Github
    # Actions be able to provision build/test environments quickly...
    #
    inherit (rust.packages.holochain-rsm) cargo;
    inherit (darwin.apple_sdk.frameworks) CoreServices Security;
in

{
  #
  # An example Node application build procedure; nix-shell's shell.nix will
  # inherit these buildInputs?  Once we recover dnaPackages and buildDna, we
  # should be able to build DNAs again in a functionally defined way.
  #
  elemental-chat = stdenv.mkDerivation rec {
    name = "elemental-chat";
    src = gitignoreSource ./.;

    buildInputs = [
      holochain
      lair-keystore
      cargo
    ];

    nativeBuildInputs = [
      nodejs
    ];
  };
}

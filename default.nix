let
  holonixPath = builtins.fetchTarball {
    url = "https://github.com/holochain/holonix/archive/c124fe5adb4348c04e0d69ef4242dd5cdb536b20.tar.gz";
    sha256 = "sha256:04s6ixq647ndq6zlynnvk0y5j4az7cg7m8jix5a2bmixlcvla6wi";
  };
  holonix = import (holonixPath) {
    includeHolochainBinaries = true;
    holochainVersionId = "custom";

    holochainVersion = {
     rev = "bd89d55e397baf7876099f600db997c89dd70fb6";
     sha256 = "1nbq8qq4hzww5460khhv5ihj76bsnfqs306dcyknb2rq2firl1m1";
     cargoSha256 = "sha256:1hbdcks7hkd3924wjy9qkizyn2hvhdvm14m23sjl9nn3xygj8w7w";
     bins = {
       holochain = "holochain";
       hc = "hc";
     };
    };
    holochainOtherDepsNames = ["lair-keystore"];
  };
  nixpkgs = holonix.pkgs;
in nixpkgs.mkShell {
  inputsFrom = [ holonix.main ];
  buildInputs = with nixpkgs; [
    binaryen
  ];
}

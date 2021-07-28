let
  holonixPath = builtins.fetchTarball {
    url = "https://github.com/holochain/holonix/archive/c124fe5adb4348c04e0d69ef4242dd5cdb536b20.tar.gz";
    sha256 = "sha256:04s6ixq647ndq6zlynnvk0y5j4az7cg7m8jix5a2bmixlcvla6wi";
  };
  holonix = import (holonixPath) {
    includeHolochainBinaries = true;
    holochainVersionId = "custom";

    holochainVersion = {
     rev = "c5dbdf28825927106bc32d186dd54f20d35df468";
     sha256 = "0spkrpl8bcpckbwpvl3b17izqd7yh88gdrc7iianzl3phh7kkwz6";
     cargoSha256 = "sha256:086snrrywkrdzr1hngra4vib2c3ci7wa1782w7mb5ya5bpa2m28h";
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

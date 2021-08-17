let
  holonixPath = builtins.fetchTarball {
    url = "https://github.com/holochain/holonix/archive/cedfc2453cfa795a0344acd6f5fb302362e18fc5.tar.gz";
    sha256 = "sha256:004s7lkfhb5lg5292b80byx8b8zdm8lc3g0ldhfx9biqdg6m9agp";
  };
  holonix = import (holonixPath) {
    includeHolochainBinaries = true;
    holochainVersionId = "custom";

    holochainVersion = {
     rev = "1f0082e442cd20f3b0a097a13384ffca1c0758b5";
     sha256 = "0lwzdvdslm64pfw87fgppndvkl5hv3qzxdmcpdcdpxnc6i65pyhr";
     cargoSha256 = "sha256:16gv3yil2my30fl2bqadypd5rbbvsksdd7yivysq6brc2xwzkvin";
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

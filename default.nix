let
  holonixPath = builtins.fetchTarball {
    url = "https://github.com/holochain/holonix/archive/cedfc2453cfa795a0344acd6f5fb302362e18fc5.tar.gz";
    sha256 = "sha256:004s7lkfhb5lg5292b80byx8b8zdm8lc3g0ldhfx9biqdg6m9agp";
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

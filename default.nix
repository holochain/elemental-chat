let
  holonixPath = builtins.fetchTarball {
    url = "https://github.com/holochain/holonix/archive/cedfc2453cfa795a0344acd6f5fb302362e18fc5.tar.gz";
    sha256 = "sha256:004s7lkfhb5lg5292b80byx8b8zdm8lc3g0ldhfx9biqdg6m9agp";
  };
  holonix = import (holonixPath) {
    includeHolochainBinaries = true;
    holochainVersionId = "custom";

    holochainVersion = {
     rev = "0d1c06630000beb9e3c5f6e54ae85cbe9ffa484a";
     sha256 = "1shjal90zxrqyz9abzw2a3m4p54g26pj752bs0cxicc71vwgzhy0";
     cargoSha256 = "sha256:0nvqimhfal82imvhrc5ahp1khp7779n8i3zkcgf2yps9b0myxr11";
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

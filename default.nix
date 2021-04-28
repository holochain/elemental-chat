let
  holonixPath = builtins.fetchTarball {
    url = "https://github.com/holochain/holonix/archive/87ad95a9a0b08deea64ad77ac14a68a7f12cff52.tar.gz";
    sha256 = "0fvbbaps9aggqkjr00b3b331avh0fjb2b8gn07yglshsgix7wrhh";
  };
  holonix = import (holonixPath) {
    includeHolochainBinaries = true;
    holochainVersionId = "custom";

    holochainVersion = {
     rev = "a1ae76ecc1fc7dbd645fee3a8cb0df9f610be983";
     sha256 = "04kha2vxzh3ml452xsdz40f3jbchlad0lf741862n56x4np2spa2";
     cargoSha256 = "19pjrzkzkvq4rhz66dx8h41wf5swi71ycvzna0fx25i2vcpfnfaa";
     bins = {
       holochain = "holochain";
       hc = "hc";
     };
    };
    holochainOtherDepsNames = ["lair-keystore"];
  };
in holonix.main

let
  holonixPath = builtins.fetchTarball {
    url = "https://github.com/holochain/holonix/archive/3e94163765975f35f7d8ec509b33c3da52661bd1.tar.gz";
    sha256 = "sha256:07sl281r29ygh54dxys1qpjvlvmnh7iv1ppf79fbki96dj9ip7d2";
  };
  holonix = import (holonixPath) {
    includeHolochainBinaries = true;
    holochainVersionId = "custom";

    holochainVersion = {
     rev = "30045ec5269f2b080bc35002ee01f764d2a95b5e";
     sha256 = "0zbjd6gwf4409q8j7i71562gfr37l4jwymg89an717vqmsaj2hw3";
     cargoSha256 = "sha256:0kafaxv10vlnp6bxyv3y57m58hy40qn12j9mc0p8mhznah994iha";
     bins = {
       holochain = "holochain";
       hc = "hc";
     };
    };
    holochainOtherDepsNames = ["lair-keystore"];
  };
in holonix.main

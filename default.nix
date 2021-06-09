let
  holonixPath = builtins.fetchTarball {
    url = "https://github.com/holochain/holonix/archive/3e94163765975f35f7d8ec509b33c3da52661bd1.tar.gz";
    sha256 = "sha256:07sl281r29ygh54dxys1qpjvlvmnh7iv1ppf79fbki96dj9ip7d2";
  };
  holonix = import (holonixPath) {
    includeHolochainBinaries = true;
    holochainVersionId = "custom";

    holochainVersion = {
     rev = "33dcca03d6113eb8838df6909a46b12272dde0b7";
     sha256 = "0kn1y5bz58garrqvmk8yr151f664llfckwl0li9bckcayyccvsk9";
     cargoSha256 = "sha256:1fml9bngmp9dsgcrly0vavvdgii6fkcj4mjyakbck45kd8xx449d";
     bins = {
       holochain = "holochain";
       hc = "hc";
     };
    };
    holochainOtherDepsNames = ["lair-keystore"];
  };
in holonix.main

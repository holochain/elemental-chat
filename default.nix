let
  holonixPath = builtins.fetchTarball {
    url = "https://github.com/holochain/holonix/archive/3e94163765975f35f7d8ec509b33c3da52661bd1.tar.gz";
    sha256 = "sha256:07sl281r29ygh54dxys1qpjvlvmnh7iv1ppf79fbki96dj9ip7d2";
  };
  holonix = import (holonixPath) {
    includeHolochainBinaries = true;
    holochainVersionId = "custom";

    holochainVersion = {
     rev = "cfa7041e887921661206dc41f283167b9f62ed98";
     sha256 = "1k8j9xm65ynrb5lwg6wb8hlngwwhl0kqqnkqn953lfkifwcwba3v";
     cargoSha256 = "1gv2zhslraxiq8v5644mxb2d1sl0md6i79rsjppcf0gi7n6pd6zi";
     bins = {
       holochain = "holochain";
       hc = "hc";
     };
    };
    holochainOtherDepsNames = ["lair-keystore"];
  };
in holonix.main

let
  holonixPath = builtins.fetchTarball {
    url = "https://github.com/holochain/holonix/archive/3e94163765975f35f7d8ec509b33c3da52661bd1.tar.gz";
    sha256 = "sha256:07sl281r29ygh54dxys1qpjvlvmnh7iv1ppf79fbki96dj9ip7d2";
  };
  holonix = import (holonixPath) {
    includeHolochainBinaries = true;
    holochainVersionId = "custom";

    holochainVersion = {
     rev = "f4f1c074020a4e97ea4fc877c5b102cd347d72a3";
     sha256 = "005inh7l3kf8sd1cqda47k0bhn9pq4nycl6prh4wz03683hcdvq5";
     cargoSha256 = "1phciznm7xs82s0hqv49lw2q4k5bpz8x60czj5cpb9964qncik3q";
     bins = {
       holochain = "holochain";
       hc = "hc";
     };
    };
    holochainOtherDepsNames = ["lair-keystore"];
  };
in holonix.main

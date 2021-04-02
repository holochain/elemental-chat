let
  holonixPath = builtins.fetchTarball {
    url = "https://github.com/holochain/holonix/archive/87ad95a9a0b08deea64ad77ac14a68a7f12cff52.tar.gz";
    sha256 = "0fvbbaps9aggqkjr00b3b331avh0fjb2b8gn07yglshsgix7wrhh";
  };
  holonix = import (holonixPath) {
    includeHolochainBinaries = true;
    holochainVersionId = "custom";

    holochainVersion = {
     rev = "e157f64b67d5352cfba396b8088f90073879c8e0";
     sha256 = "sha256:1gffdlk2hmdjn54fg152yqfhmqqjcvfj2350ypnrir5cgjwj0369";
     cargoSha256 = "sha256:1yqk7vhw0j8y6zv8aimlr8phavsbzwxllqd1pvcmnkjmr1c1kzrm";
     bins = {
       holochain = "holochain";
       hc = "hc";
     };
    };
    holochainOtherDepsNames = ["lair-keystore"];
  };
in holonix.main

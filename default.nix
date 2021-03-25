let
  holonixPath = builtins.fetchTarball {
    url = "https://github.com/holochain/holonix/archive/87ad95a9a0b08deea64ad77ac14a68a7f12cff52.tar.gz";
    sha256 = "0fvbbaps9aggqkjr00b3b331avh0fjb2b8gn07yglshsgix7wrhh";
  };
  holonix = import (holonixPath) {
    includeHolochainBinaries = true;
    holochainVersionId = "custom";

    holochainVersion = {
     rev = "a82372a62d46a503e48f345360d0fb18cc5822d1";
     sha256 = "1flz7cqm2rpnchsnfai83vw65m57x69ajrhd8v5zgnjcnc1plymx";
     cargoSha256 = "1fpkb3ga1f4xvgazpd31v45y4gbf6ss1y1rjzqx300vlp99237f9";
     bins = {
       holochain = "holochain";
       hc = "hc";
     };
    };
    holochainOtherDepsNames = ["lair-keystore"];
  };
in holonix.main

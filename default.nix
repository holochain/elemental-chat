let
  holonixPath = builtins.fetchTarball {
    url = "https://github.com/holochain/holonix/archive/87ad95a9a0b08deea64ad77ac14a68a7f12cff52.tar.gz";
    sha256 = "0fvbbaps9aggqkjr00b3b331avh0fjb2b8gn07yglshsgix7wrhh";
  };
  holonix = import (holonixPath) {
    includeHolochainBinaries = true;
    holochainVersionId = "custom";

    holochainVersion = {
     rev = "8d6c4cd29bd17e8224aeffb87dc03eaf3ff33508";
     sha256 = "0ds7w342b23rgmr2kwh8qk9ccb16sflr7jd1hj1n3nkfj1m3xa52";
     cargoSha256 = "19pjrzkzkvq4rhz66dx8h41wf5swi71ycvzna0fx25i2vcpfnf35";
     bins = {
       holochain = "holochain";
       hc = "hc";
     };
    };
    holochainOtherDepsNames = ["lair-keystore"];
  };
in holonix.main

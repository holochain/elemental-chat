let
  holonixPath = builtins.fetchTarball {
    url = "https://github.com/holochain/holonix/archive/9a1f11bacf0a4a8d5cfb83b449df5f726c569c7c.tar.gz";
    sha256 = "0j4v354rlipifw2ibydr02nv6bwm33vv63197srswkvv6wi6dr9c";
  };
  holonix = import (holonixPath) {
    includeHolochainBinaries = true;
    holochainVersionId = "custom";

    holochainVersion = {
     rev = "59c8632876795125a8d0a3770b2fd6733529bfcd";
     sha256 = "1j56rlkw755xn14zk8d5zaj87mkk3iz44p2zk3qx1mr9hdrfcqbh";
     cargoSha256 = "0pr0qjidqycfv367cdmhk5jgbf5vvn0a3b1ds477x28f9y86aac1";
     bins = {
       holochain = "holochain";
       hc = "hc";
     };
    };
    holochainOtherDepsNames = ["lair-keystore"];
  };
in holonix.main

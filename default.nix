let
  holonixPath = builtins.fetchTarball {
    url = "https://github.com/holochain/holonix/archive/9a1f11bacf0a4a8d5cfb83b449df5f726c569c7c.tar.gz";
    sha256 = "sha256:0j4v354rlipifw2ibydr02nv6bwm33vv63197srswkvv6wi6dr9c";
  };
  holonix = import (holonixPath) {
    includeHolochainBinaries = true;
    holochainVersionId = "custom";

    holochainVersion = {
     rev = "3bd9181ea35c32993d1550591fd19720b31065f6";
     sha256 = "1sbdcxddpa33gqmly4x5gz2l4vhmab8hwjngpibmqfr1ga6v56wv";
     cargoSha256 = "sha256:1ls4524519jqqw42q2jsj5bxcmf1vn8hmgjzywffca2ycmvd788p";
     bins = {
       holochain = "holochain";
       hc = "hc";
     };
    };
    holochainOtherDepsNames = ["lair-keystore"];
  };
in holonix.main

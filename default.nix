let
  holonixPath = builtins.fetchTarball {
    url = "https://github.com/holochain/holonix/archive/014d28000c8ed021eb84000edfe260c22e90af8c.tar.gz";
    sha256 = "sha256:0hl5xxxjg2a6ymr44rf5dfvsb0c33dq4s6vibva6yb76yvl6gwfi";
  };
  holonix = import (holonixPath) {
    includeHolochainBinaries = true;
    holochainVersionId = "custom";

    holochainVersion = {
     rev = "363af6d8af8d18e4616f6aa56ad4d1f0fabaafb7";
     sha256 = "0ssjhang6zljs0zrph998zj7582rf0vdb45p855awa7fmzpd4kfa";
     cargoSha256 = "sha256:0y72lm5b0fl9anb2z9pcx1i3shqdlckz04zx3phc084hbzpig4cq";
     bins = {
       holochain = "holochain";
       hc = "hc";
     };
    };
    holochainOtherDepsNames = ["lair-keystore"];
  };
  nixpkgs = holonix.pkgs;
in nixpkgs.mkShell {
  inputsFrom = [ holonix.main ];
  buildInputs = with nixpkgs; [
    binaryen
  ];
}

let
  holonixPath = builtins.fetchTarball {
    url = "https://github.com/holochain/holonix/archive/014d28000c8ed021eb84000edfe260c22e90af8c.tar.gz";
    sha256 = "sha256:0hl5xxxjg2a6ymr44rf5dfvsb0c33dq4s6vibva6yb76yvl6gwfi";
  };
  holonix = import (holonixPath) {
    includeHolochainBinaries = true;
    holochainVersionId = "custom";

    holochainVersion = {
     rev = "8c85ba9629631a17b59b77b9a5a0d5f24c700d11";
     sha256 = "1rflpzkb3vpvkx833mdm2mp2h3hgd7f2hk9444vjrhbvjgqr8i3n";
     cargoSha256 = "19h9lh26sayg3kpl6mcyidgg4lwh2xpiyk9157p8phsia7rn4bzr";
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

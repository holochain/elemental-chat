let
  holonixPath = builtins.fetchTarball https://github.com/holochain/holonix/archive/6ae8ffb8e5c1a1faa4f4e1af8a9f7139b2ce0f3c.tar.gz;
  holonix = import (holonixPath) {
    includeHolochainBinaries = true;
    holochainVersionId = "custom";

    holochainVersion = {
      rev = "0d1c06630000beb9e3c5f6e54ae85cbe9ffa484a";
      sha256 = "1shjal90zxrqyz9abzw2a3m4p54g26pj752bs0cxicc71vwgzhy0";
      cargoSha256 = "sha256:0nvqimhfal82imvhrc5ahp1khp7779n8i3zkcgf2yps9b0myxr11";
      bins = {
        holochain = "holochain";
        hc = "hc";
      };

      lairKeystoreHashes = {
        sha256 = "1jiz9y1d4ybh33h1ly24s7knsqyqjagsn1gzqbj1ngl22y5v3aqh";
        cargoSha256 = "0agykcl7ysikssfwkjgb3hfw6xl0slzy38prc4rnzvagm5wd1jjv";
      };
    };
  };
  nixpkgs = holonix.pkgs;
in nixpkgs.mkShell {
  inputsFrom = [ holonix.main ];
  buildInputs = with nixpkgs; [
    binaryen
  ];
}
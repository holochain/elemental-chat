let
  holonixPath = builtins.fetchTarball {
    url = "https://github.com/holochain/holonix/archive/55a5eef58979fb6bc476d8c3e0c028cdeb1b5421.tar.gz";
    sha256 = "sha256:0q6d0rql1pyy93xw1c8s28jjjcgk1zgwxwixsp9z5r4w2ihaz3zg";
  };
  holonix = import (holonixPath) {
    includeHolochainBinaries = true;
    holochainVersionId = "custom";

    holochainVersion = {
     rev = "0d1c06630000beb9e3c5f6e54ae85cbe9ffa484a";
     sha256 = "1shjal90zxrqyz9abzw2a3m4p54g26pj752bs0cxicc71vwgzhy0";
     cargoSha256 = "0nvqimhfal82imvhrc5ahp1khp7779n8i3zkcgf2yps9b0myxr11";
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
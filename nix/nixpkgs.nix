let
  mozNixpkgsRev = "c985206e160204707e15a45f0b9df4221359d21c";
  mozNixpkgs = builtins.fetchTarball {
    url = "https://github.com/mozilla/nixpkgs-mozilla/archive/${mozNixpkgsRev}.tar.gz";
    sha256 = "0k0p3nfzr3lfgp1bb52bqrbqjlyyiysf8lq2rnrmn759ijxy2qmq";
  };
  mozNixpkgsOverlays = import mozNixpkgs;
  nixpkgsRev = "8070a6333f3fc41ef93c2b0e07f999459615cc8d"; # Following nixpkgs-unstable
  nixpkgs = builtins.fetchTarball {
    url = "https://github.com/NixOS/nixpkgs-channels/archive/${nixpkgsRev}.tar.gz";
    sha256 = "0v6nycl7lzr1kdsy151j10ywhxvlb4dg82h55hpjs1dxjamms9i3";
  };
in
  import (nixpkgs) { overlays = [ mozNixpkgsOverlays ]; config = {}; }

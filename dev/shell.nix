let
  nixpkgs = import <nixpkgs> {};
  libs = with nixpkgs.pkgs; [
    openssl

    xorg.libX11
    xorg.libxcb
    xorg.libXrandr
    xorg.libXcursor
    xorg.libXi

    libGL
    SDL2
  ];
  ldpath = with nixpkgs.pkgs; nixpkgs.lib.makeLibraryPath libs;
  pkgcfgpath = with nixpkgs.pkgs; nixpkgs.lib.makeSearchPathOutput "lib" "lib/pkgconfig" [
    SDL2.dev
  ];

  self = with nixpkgs; nixpkgs.stdenv.mkDerivation rec {
    name = "transpi-${version}-builder0";
    version = "rev-4c34dd";

    # Haven't upload yet, but usable for nix-shell.


    nativeBuildInputs = [
        clang
        pkgconfig
    ];
    buildInputs = libs;

    doCheck = false;

    shellHooks = ''
      #LIBRARY_PATH="$(ldpath}";
      #export LIBRARY_PATH;

      LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${ldpath}";
      export LD_LIBRARY_PATH;

      PKG_CONFIG_PATH="$PKG_CONFIG_PATH:${pkgcfgpath}";
      export PKG_CONFIG_PATH;

      PATH=$PATH:~/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin
    '';
  };
in self

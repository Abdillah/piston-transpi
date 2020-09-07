let
  nixpkgs = import <nixpkgs> {};
  ldpath = with nixpkgs.pkgs; nixpkgs.lib.makeLibraryPath [
    openssl

    xorg.libX11
    xorg.libxcb
    xorg.libXrandr
    xorg.libXcursor
    xorg.libXi

    libGL
  ];

  self = with nixpkgs; nixpkgs.stdenv.mkDerivation rec {
    name = "transpi-${version}-builder0";
    version = "rev-4c34dd";

    # Haven't upload yet, but usable for nix-shell.


    nativeBuildInputs = [
        clang
        pkgconfig
    ];

    doCheck = false;

    shellHooks = ''
      LD_LIBRARY_PATH=$LD_LIBRARY_PATH:${ldpath}
      export LD_LIBRARY_PATH

      PATH=$PATH:~/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin
    '';
  };
in self

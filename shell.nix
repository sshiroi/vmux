{ pkgs ? import <nixpkgs> {}}:
pkgs.mkShell {
  buildInputs = with pkgs; [
    rustup pkg-config clang libclang libbluray bzip2 libclang fuse gcc vlc mpv lsdvd vbindiff
    
    ffms
    #(ffms.overrideAttrs (prev: rec {
    #  version = "git";
    #  src = fetchFromGitHub {
    #    owner = "FFMS";
    #    repo = "ffms2";
    #    rev = "ff61bca13e2c5fb99c0450620c9244f415ec29c4";
    #    sha256 = "sha256-t7AQTHr6iSLBfrdMa6CxeGWrLFi3gm+pxjbbpNUv+3Y=";
    #  };
    #}))
    
  ];
  #shellHook = ''
  #export LIBCLANG_PATH=${pkgs.llvmPackages.libclang}/lib
  #'';

  shellHook = ''export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath [
    pkgs.alsa-lib
    pkgs.xorg.libX11
    pkgs.xorg.libXcursor
    pkgs.xorg.libXrandr
    pkgs.xorg.libXi
    pkgs.libglvnd
    pkgs.udev
    pkgs.vulkan-loader

  ]}"'';

  LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib";
}

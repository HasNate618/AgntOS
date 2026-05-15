{ lib, stdenvNoCC }:

stdenvNoCC.mkDerivation {
  pname = "agntos-wallpapers";
  version = "1.0";

  src = ./wallpapers;

  dontBuild = true;

  installPhase = ''
    mkdir -p $out/share/wallpapers/agntos
    cp -r $src/* $out/share/wallpapers/agntos/
  '';

  meta = {
    description = "AgntOS curated wallpaper collection";
    platforms = lib.platforms.linux;
  };
}

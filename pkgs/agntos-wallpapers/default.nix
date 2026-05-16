{ lib, stdenvNoCC }:

stdenvNoCC.mkDerivation {
  pname = "agntos-wallpapers";
  version = "1.0";
  dontUnpack = true;
  dontBuild = true;

  installPhase = ''
    mkdir -p $out/share/wallpapers/agntos
    cp -r ${./wallpapers}/* $out/share/wallpapers/agntos/
  '';

  meta = {
    description = "AgntOS curated wallpaper collection";
    platforms = lib.platforms.linux;
  };
}

{ lib, stdenvNoCC, fetchFromGitHub }:

stdenvNoCC.mkDerivation rec {
  pname = "winsur-kde";
  version = "unstable-2025-02-03";

  src = fetchFromGitHub {
    owner = "yeyushengfan258";
    repo = "WinSur-kde";
    rev = "ef2a65c2fbeee3f5e7d5cc7472646b370ff4050e";
    sha256 = "1sjqfwyqzb5iwacxkxnxp3kbs68ild1babmvpbl1nm8sxdxar6qf";
  };

  dontBuild = true;

  installPhase = ''
    # Aurorae (window decorations)
    mkdir -p $out/share/aurorae/themes
    cp -r $src/aurorae/* $out/share/aurorae/themes/

    # Color schemes
    mkdir -p $out/share/color-schemes
    cp -r $src/color-schemes/*.colors $out/share/color-schemes/

    # Kvantum themes
    mkdir -p $out/share/Kvantum
    cp -r $src/Kvantum/* $out/share/Kvantum/

    # Plasma desktop themes
    mkdir -p $out/share/plasma/desktoptheme
    cp -r $src/plasma/desktoptheme/* $out/share/plasma/desktoptheme/

    # Plasma look-and-feel
    mkdir -p $out/share/plasma/look-and-feel
    cp -r $src/plasma/look-and-feel/* $out/share/plasma/look-and-feel/

    # Wallpapers
    mkdir -p $out/share/wallpapers
    cp -r $src/wallpaper/* $out/share/wallpapers/
  '';

  meta = {
    description = "WinSur kde - Windows 11-inspired KDE Plasma theme";
    homepage = "https://github.com/yeyushengfan258/WinSur-kde";
    license = lib.licenses.gpl3Only;
    platforms = lib.platforms.linux;
  };
}

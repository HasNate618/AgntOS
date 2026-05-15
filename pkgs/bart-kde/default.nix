{ lib, stdenvNoCC, fetchFromGitLab }:

stdenvNoCC.mkDerivation rec {
  pname = "bart-kde";
  version = "1.2";

  src = fetchFromGitLab {
    owner = "jomada";
    repo = "bart";
    rev = "main";
    sha256 = "ip4Nk3EUGyjxcvng+IFEh8XP3A8aUFymqiXJCqHYDDM=";
  };

  dontBuild = true;

  installPhase = ''
    # Aurorae (window decorations)
    mkdir -p $out/share/aurorae/themes
    cp -r $src/aurorae/Bart $out/share/aurorae/themes/

    # Color scheme
    mkdir -p $out/share/color-schemes
    cp $src/color-schemes/Bart.colors $out/share/color-schemes/

    # Plasma desktop theme
    mkdir -p $out/share/plasma/desktoptheme
    cp -r $src/plasma/Bart $out/share/plasma/desktoptheme/

    # Plasma look-and-feel
    mkdir -p $out/share/plasma/look-and-feel
    cp -r $src/global/Bart $out/share/plasma/look-and-feel/

    # Konsole theme
    mkdir -p $out/share/konsole
    cp $src/konsole/Bart.colorscheme $out/share/konsole/
  '';

  meta = {
    description = "Bart Desktop Design - KDE Plasma theme";
    homepage = "https://gitlab.com/jomada/bart";
    license = lib.licenses.gpl3Only;
    platforms = lib.platforms.linux;
  };
}

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

    # Color scheme — darker background + AgntOS orange accent
    mkdir -p $out/share/color-schemes
    sed \
      's/BackgroundNormal=51,48,48/BackgroundNormal=8,8,10/g; s/BackgroundNormal=21,20,20/BackgroundNormal=8,8,10/g; s/BackgroundAlternate=21,20,20/BackgroundAlternate=10,10,13/g; s/BackgroundAlternate=51,48,48/BackgroundAlternate=10,10,13/g; s/BackgroundNormal=217,90,60/BackgroundNormal=255,125,75/g; s/BackgroundAlternate=217,90,60/BackgroundAlternate=255,125,75/g; s/ForegroundNormal=237,240,242/ForegroundNormal=220,218,210/g; s/ForegroundNormal=201,212,230/ForegroundNormal=210,208,200/g; s/ForegroundNormal=183,186,195/ForegroundNormal=200,198,190/g' \
      $src/color-schemes/Bart.colors > $out/share/color-schemes/Bart.colors

    # Plasma desktop theme — copy all, then patch specific SVGs
    mkdir -p $out/share/plasma/desktoptheme
    cp -r $src/plasma/Bart $out/share/plasma/desktoptheme/
    for f in panel-background translucentbackground background; do
      DOTFILE="$out/share/plasma/desktoptheme/Bart/widgets/''${f}.svgz"
      [ -f "$DOTFILE" ] || continue
      gzip -dc < "$DOTFILE" | sed \
        's/stop-opacity:0.49803922/stop-opacity:0.08/g; s/stop-opacity:0.81600001/stop-opacity:0.25/g; s/stop-opacity:0.86000001/stop-opacity:0.2/g; s/stop-opacity:0.875/stop-opacity:0.3/g; s/stop-opacity:0.81589954/stop-opacity:0.2/g' \
        | gzip > /tmp/patched-''${f}.svgz
      chmod u+w "$DOTFILE"
      cp /tmp/patched-''${f}.svgz "$DOTFILE"
    done

    # Plasma look-and-feel
    mkdir -p $out/share/plasma/look-and-feel
    cp -r $src/global/Bart $out/share/plasma/look-and-feel/

    # Konsole theme
    mkdir -p $out/share/konsole
    cp $src/konsole/Bart.colorscheme $out/share/konsole/
  '';

  meta = {
    description = "AgntOS-customized Bart Desktop Design - KDE Plasma theme";
    homepage = "https://gitlab.com/jomada/bart";
    license = lib.licenses.gpl3Only;
    platforms = lib.platforms.linux;
  };
}

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
      's/BackgroundNormal=51,48,48/BackgroundNormal=25,25,30/g; s/BackgroundNormal=21,20,20/BackgroundNormal=25,25,30/g; s/BackgroundAlternate=21,20,20/BackgroundAlternate=30,30,35/g; s/BackgroundAlternate=51,48,48/BackgroundAlternate=30,30,35/g; s/BackgroundNormal=217,90,60/BackgroundNormal=245,124,72/g; s/BackgroundAlternate=217,90,60/BackgroundAlternate=245,124,72/g; s/ForegroundNormal=237,240,242/ForegroundNormal=220,218,210/g; s/ForegroundNormal=201,212,230/ForegroundNormal=210,208,200/g; s/ForegroundNormal=183,186,195/ForegroundNormal=200,198,190/g' \
      $src/color-schemes/Bart.colors > $out/share/color-schemes/Bart.colors

    # Build Plasma desktop theme: copy Bart, then replace SVGs with clean ones
    mkdir -p $out/share/plasma/desktoptheme
    cp -r $src/plasma/Bart $out/share/plasma/desktoptheme/
    chmod -R u+w "$out/share/plasma/desktoptheme/Bart"
    # Create variant directories (Plasma 6 uses opaque/translucent/solid)
    for variant in opaque translucent solid; do
      mkdir -p "$out/share/plasma/desktoptheme/Bart/''${variant}/widgets"
    done
    for svg in panel-background translucentbackground background; do
      # Simple SVG: dark semi-transparent background
      gzip > "$out/share/plasma/desktoptheme/Bart/widgets/''${svg}.svgz" << 'SVGEOF'
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10">
  <rect width="10" height="10" fill="#1a1a20" opacity="0.25"/>
</svg>
SVGEOF
      for variant in opaque translucent solid; do
        cp "$out/share/plasma/desktoptheme/Bart/widgets/''${svg}.svgz" \
          "$out/share/plasma/desktoptheme/Bart/''${variant}/widgets/"
      done
    done
    # Opaque variant override: fully opaque black
    for svg in panel-background translucentbackground; do
      gzip > "$out/share/plasma/desktoptheme/Bart/opaque/widgets/''${svg}.svgz" << 'SVGEOF'
<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10">
  <rect width="10" height="10" fill="#000000" opacity="1"/>
</svg>
SVGEOF
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

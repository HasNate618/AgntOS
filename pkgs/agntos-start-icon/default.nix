{ lib, stdenvNoCC, agntos-branding }:

stdenvNoCC.mkDerivation {
  pname = "agntos-start-icon";
  version = "0.1.0";
  dontUnpack = true;

  installPhase = ''
    mkdir -p $out/share/icons/agntos-start/places/64
    mkdir -p $out/share/icons/agntos-start/places/symbolic

    # Use our logo for the start button at all sizes
    for size in 16 22 24 32 48 64 96 128 256; do
      mkdir -p $out/share/icons/agntos-start/places/$size
      cp ${agntos-branding}/share/agntos/logos/agntos.svg \
        $out/share/icons/agntos-start/places/$size/start-here-kde-plasma.svg
      cp ${agntos-branding}/share/agntos/logos/agntos.svg \
        $out/share/icons/agntos-start/places/$size/start-here-kde.svg
    done
    cp ${agntos-branding}/share/agntos/logos/agntos.svg \
      $out/share/icons/agntos-start/places/symbolic/start-here-symbolic.svg

    # Also set as applications icon for good measure
    mkdir -p $out/share/icons/agntos-start/apps/64
    cp ${agntos-branding}/share/agntos/logos/agntos.svg \
      $out/share/icons/agntos-start/apps/64/agntos.svg

    # index.theme — inherit all other icons from Kora
    cat > $out/share/icons/agntos-start/index.theme << EOF
[Icon Theme]
Name=AgntOS
Inherits=kora
Directories=places/64,places/symbolic
EOF
  '';

  meta = {
    description = "AgntOS start button icon — replaces Plasma logo";
    license = lib.licenses.mit;
  };
}

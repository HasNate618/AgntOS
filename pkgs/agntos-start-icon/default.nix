{ lib, stdenvNoCC, agntos-branding }:

stdenvNoCC.mkDerivation {
  pname = "agntos-start-icon";
  version = "0.1.0";
  dontUnpack = true;

  installPhase = ''
    mkdir -p $out/share/icons/agntos-start/places/16
    mkdir -p $out/share/icons/agntos-start/places/22
    mkdir -p $out/share/icons/agntos-start/places/24
    mkdir -p $out/share/icons/agntos-start/places/32
    mkdir -p $out/share/icons/agntos-start/places/48
    mkdir -p $out/share/icons/agntos-start/places/64
    mkdir -p $out/share/icons/agntos-start/places/96
    mkdir -p $out/share/icons/agntos-start/places/128
    mkdir -p $out/share/icons/agntos-start/places/256
    mkdir -p $out/share/icons/agntos-start/places/symbolic

    # Use our logo for the start button at all sizes
    for size in 16 22 24 32 48 64 96 128 256; do
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
    # Size entries required for GTK/Qt icon cache to index correctly
    cat > $out/share/icons/agntos-start/index.theme << EOF
[Icon Theme]
Name=AgntOS
Comment=AgntOS start button icon
Inherits=kora
Example=start-here-kde-plasma
FollowsColorScheme=true
Directories=places/16,places/22,places/24,places/32,places/48,places/64,places/96,places/128,places/256,places/symbolic

[places/16]
Size=16
Type=Fixed
Context=Places

[places/22]
Size=22
Type=Fixed
Context=Places

[places/24]
Size=24
Type=Fixed
Context=Places

[places/32]
Size=32
Type=Fixed
Context=Places

[places/48]
Size=48
Type=Fixed
Context=Places

[places/64]
Size=64
Type=Fixed
Context=Places

[places/96]
Size=96
Type=Fixed
Context=Places

[places/128]
Size=128
Type=Fixed
Context=Places

[places/256]
Size=256
Type=Fixed
Context=Places

[places/symbolic]
Size=16
MinSize=16
MaxSize=512
Type=Scalable
Context=Places
EOF
  '';

  meta = {
    description = "AgntOS start button icon — replaces Plasma logo";
    license = lib.licenses.mit;
  };
}

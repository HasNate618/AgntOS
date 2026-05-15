{ lib, stdenvNoCC, callPackage }:

stdenvNoCC.mkDerivation {
  pname = "agntos-branding";
  version = "0.1.0";
  dontUnpack = true;

  installPhase = ''
    mkdir -p $out/share/wallpapers/agntos
    install -m 644 ${./wallpapers/default.png} $out/share/wallpapers/agntos/default.png
    install -m 644 ${./wallpapers/install.png} $out/share/wallpapers/agntos/install.png

    mkdir -p $out/share/agntos
    install -m 644 ${./fastfetch-config.jsonc} $out/share/agntos/fastfetch-config.jsonc

    mkdir -p $out/share/agntos/logos
    install -m 644 ${./logo/agntos.svg} $out/share/agntos/logos/agntos.svg
    install -m 644 ${./logo/agntos.png} $out/share/agntos/logos/agntos.png
    install -m 644 ${./logo/agntos-ascii.txt} $out/share/agntos/logo.txt

    # Plymouth theme
    mkdir -p $out/share/plymouth/themes/agntos
    install -m 644 ${./logo/agntos.png} $out/share/plymouth/themes/agntos/logo.png
    install -m 644 ${./plymouth/bar.png} $out/share/plymouth/themes/agntos/bar.png
    install -m 644 ${./plymouth/bar-bg.png} $out/share/plymouth/themes/agntos/bar-bg.png
    install -m 644 ${./plymouth/agntos.script} $out/share/plymouth/themes/agntos/agntos.script
    # Generate .plymouth with correct store paths
    sed "s|@out@|$out/share/plymouth/themes/agntos|g" ${./plymouth/agntos.plymouth.in} \
      > $out/share/plymouth/themes/agntos/agntos.plymouth
  '';

  meta = {
    description = "AgntOS branding assets: wallpapers, logos, configs";
    license = lib.licenses.mit;
  };
}

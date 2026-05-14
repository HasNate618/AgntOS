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

    mkdir -p $out/etc/
    install -m 644 ${./logo/agntos-ascii.txt} $out/share/agntos/logo.txt
  '';

  meta = {
    description = "AgntOS branding assets: wallpapers, logos, configs";
    license = lib.licenses.mit;
  };
}

{ lib, stdenvNoCC, fetchurl, unzip }:

let
  fonts = {
    syne = fetchurl {
      url = "https://github.com/google/fonts/raw/main/ofl/syne/Syne%5Bwght%5D.ttf";
      sha256 = "ce5ac77142a65cab2248a1a2ebb740b1d4d9c20b52488877d3ff664d1356104a";
    };
    plus-jakarta-sans = fetchurl {
      url = "https://github.com/google/fonts/raw/main/ofl/plusjakartasans/PlusJakartaSans%5Bwght%5D.ttf";
      sha256 = "89b3fb38aa0d275d7a731d0d817a4f1622b316b4d7fbdedcf02ee9099ff68bc8";
    };
    plus-jakarta-sans-italic = fetchurl {
      url = "https://github.com/google/fonts/raw/main/ofl/plusjakartasans/PlusJakartaSans-Italic%5Bwght%5D.ttf";
      sha256 = "9529eb888668b6a3c6dd75b6341a2fc5263fb6c9e788822e6117c29dd9e8b115";
    };
  };
in
stdenvNoCC.mkDerivation {
  pname = "agntos-fonts";
  version = "0.1.0";
  dontUnpack = true;
  installPhase = ''
    mkdir -p $out/share/fonts/truetype/agntos
    ${lib.concatStringsSep "\n" (lib.mapAttrsToList (name: path: ''
      cp ${path} $out/share/fonts/truetype/agntos/${name}.ttf
    '') fonts)}
  '';
  meta = {
    description = "AgntOS system fonts: Plus Jakarta Sans, Syne";
    license = lib.licenses.ofl;
  };
}

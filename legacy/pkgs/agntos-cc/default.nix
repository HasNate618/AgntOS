{ lib, rustPlatform, pkg-config, openssl
, webkitgtk_4_1, gtk3, glib, cairo, pango, gdk-pixbuf, atk
, libsoup_3, makeWrapper, glib-networking
, agntos-cc-frontend
}:

rustPlatform.buildRustPackage {
  pname = "agntos-cc";
  version = "0.1.0";

  src = lib.cleanSourceWith {
    filter = path: type:
      lib.cleanSourceFilter path type
      && baseNameOf path != "node_modules"
      && baseNameOf path != "target"
      && !(lib.hasSuffix ".png" (baseNameOf path) && !lib.hasInfix "/icons/" path)
      && !(lib.hasSuffix ".jpg" (baseNameOf path) && !lib.hasInfix "/icons/" path)
      && !(lib.hasSuffix ".jpeg" (baseNameOf path) && !lib.hasInfix "/icons/" path)
      && !(lib.hasSuffix ".svg" (baseNameOf path) && !lib.hasInfix "/icons/" path)
      && !lib.hasSuffix ".qcow2" (baseNameOf path);
    src = ../..;
  };

  cargoBuildFlags = [ "--manifest-path" "legacy/agntos-cc/Cargo.toml" ];

  nativeBuildInputs = [ pkg-config makeWrapper ];
  buildInputs = [
    openssl webkitgtk_4_1 gtk3 glib cairo pango
    gdk-pixbuf atk libsoup_3 glib-networking
    agntos-cc-frontend
  ];

  cargoLock = {
    lockFile = ../../Cargo.lock;
  };

  preBuild = ''
    echo ">>> [agntos-cc] copying pre-built frontend from ${agntos-cc-frontend}"
    rm -rf legacy/agntos-cc/frontend/dist
    cp -r ${agntos-cc-frontend}/dist legacy/agntos-cc/frontend/dist
    test -f legacy/agntos-cc/frontend/dist/index.html
    echo ">>> [agntos-cc] frontend ready"
  '';

  postInstall = ''
    mkdir -p $out/share/agntos
    cp -r legacy/agntos-cc/etc/agntos/AGENTS.md $out/share/agntos/
    cp -r legacy/agntos-cc/etc/agntos/extensions $out/share/agntos/

    mkdir -p $out/share/applications
    cat > $out/share/applications/agntos-cc.desktop << 'DESKTOP'
[Desktop Entry]
Name=AgntOS Control Centre
Comment=AI-native system agent GUI
Exec=agntos-cc
Icon=agntos-start
Terminal=false
Type=Application
Categories=Utility;System;
StartupNotify=true
DESKTOP
  '';

  meta = {
    description = "Tauri-based GUI for the AgntOS system agent";
    longDescription = ''
      AgntOS Control Centre is a desktop application that provides a graphical
      interface for interacting with the AgntOS system agent. It communicates
      with Pi backend via stdin/stdout RPC and provides chat, inspection, and
      system management capabilities.
    '';
    homepage = "https://agntos.dev";
    license = lib.licenses.mit;
    mainProgram = "agntos-cc";
    platforms = lib.platforms.linux;
  };
}

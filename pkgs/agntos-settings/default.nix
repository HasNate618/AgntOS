{ lib, rustPlatform, pkg-config, openssl, qt6, makeWrapper }:

rustPlatform.buildRustPackage {
  pname = "agntos-settings";
  version = "0.1.0";

  src = lib.cleanSourceWith {
    filter = path: type:
      lib.cleanSourceFilter path type
      && !lib.hasSuffix ".png" (baseNameOf path)
      && !lib.hasSuffix ".jpg" (baseNameOf path)
      && !lib.hasSuffix ".jpeg" (baseNameOf path)
      && !lib.hasSuffix ".svg" (baseNameOf path)
      && !lib.hasSuffix ".qcow2" (baseNameOf path);
    src = ../..;
  };

  cargoBuildFlags = [ "-p" "agntos-settings" ];

  nativeBuildInputs = [ pkg-config qt6.qtdeclarative qt6.qttools makeWrapper ];
  buildInputs = [ openssl qt6.qtdeclarative qt6.qtbase ];

  cargoLock = {
    lockFile = ../../Cargo.lock;
  };

  dontWrapQtApps = true;

  postInstall = ''
    qmlDir=$out/share/agntos-settings/qml
    mkdir -p $qmlDir
    cp -r crates/agntos-settings/resources/* $qmlDir/

    wrapProgram $out/bin/agntos-settings \
      --set AGNTOS_QML_DIR "$qmlDir" \
      --prefix LD_LIBRARY_PATH : "${lib.makeLibraryPath [ qt6.qtbase qt6.qtdeclarative ]}"

    mkdir -p $out/share/applications
    cat > $out/share/applications/agntos-settings.desktop << 'DESKTOP'
[Desktop Entry]
Name=AgntOS Control Center
Comment=Configure and interact with the AgntOS agent
Exec=agntos-settings
Icon=agntos-start
Terminal=false
Type=Application
Categories=Qt;System;
StartupNotify=true
DESKTOP

    # Also install the QML files to standard Qt QML path for auto-discovery
    mkdir -p $out/lib/qt-6/qml/AgntOS
    ln -sf $qmlDir/* $out/lib/qt-6/qml/AgntOS/
  '';

  meta = {
    description = "Kirigami GUI for AgntOS agent configuration";
    license = lib.licenses.mit;
    mainProgram = "agntos-settings";
    platforms = lib.platforms.linux;
  };
}

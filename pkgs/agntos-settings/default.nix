{ lib, rustPlatform, pkg-config, openssl, qt6, makeWrapper }:

rustPlatform.buildRustPackage {
  pname = "agntos-settings";
  version = "0.1.1";

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
  buildInputs = [ openssl qt6.qtdeclarative qt6.qtbase qt6.qtwayland ];

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
      --set QT_QPA_PLATFORM "wayland" \
      --set QT_QUICK_BACKEND "software" \
      --prefix LD_LIBRARY_PATH : "${lib.makeLibraryPath [ qt6.qtbase qt6.qtdeclarative qt6.qtwayland ]}" \
      --prefix QT_PLUGIN_PATH : "${lib.concatStringsSep ":" [
        "${lib.getLib qt6.qtbase}/lib/qt-6/plugins"
        "${lib.getLib qt6.qtwayland}/lib/qt-6/plugins"
      ]}" \
      --prefix QML2_IMPORT_PATH : "${lib.getLib qt6.qtdeclarative}/lib/qt-6/qml"

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

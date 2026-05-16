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

  postInstall = ''
    mkdir -p $out/share/agntos-settings/qml
    cp -r crates/agntos-settings/resources/* $out/share/agntos-settings/qml/

    wrapProgram $out/bin/agntos-settings \
      --prefix LD_LIBRARY_PATH : "${lib.makeLibraryPath [ qt6.qtbase qt6.qtdeclarative ]}"

    mkdir -p $out/share/applications
    cat > $out/share/applications/agntos-settings.desktop << 'DESKTOP'
[Desktop Entry]
Name=AgntOS Control Center
Comment=Configure and interact with the AgntOS agent
Exec=agntos-settings
Icon=system-run
Terminal=false
Type=Application
Categories=System;Settings;
Keywords=agent;ai;nixos;settings;
DESKTOP
  '';

  meta = {
    description = "Kirigami GUI for AgntOS agent configuration";
    license = lib.licenses.mit;
    mainProgram = "agntos-settings";
    platforms = lib.platforms.linux;
  };
}

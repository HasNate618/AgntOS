{ lib, rustPlatform, pkg-config, openssl, qt6, kirigami }:

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

  nativeBuildInputs = [ pkg-config qt6.qtdeclarative qt6.qttools qt6.wrapQtAppsHook ];
  buildInputs = [ openssl kirigami ];

  cargoLock = {
    lockFile = ../../Cargo.lock;
  };

  postInstall = ''
    mkdir -p $out/share/agntos-settings/qml
    cp -r crates/agntos-settings/resources/* $out/share/agntos-settings/qml/

    mkdir -p $out/share/applications
    cat > $out/share/applications/agntos-settings.desktop << 'EOF'
[Desktop Entry]
Name=AgntOS Control Center
Comment=Configure and interact with the AgntOS agent
Exec=agntos-settings
Icon=system-run
Terminal=false
Type=Application
Categories=System;Settings;
Keywords=agent;ai;nixos;settings;
EOF
  '';

  meta = {
    description = "Kirigami GUI for AgntOS agent configuration";
    longDescription = ''
      agntos-settings provides a chat-driven interface to the AgntOS agent,
      plus dashboard pages for system status, pending proposals, and audit
      log history. Communicates with agntd over a Unix domain socket.
    '';
    license = lib.licenses.mit;
    maintainers = [ ];
    mainProgram = "agntos-settings";
    platforms = lib.platforms.linux;
  };
}

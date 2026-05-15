    # Kvantum — installed but NOT forced (crashes Plasma 6 on NixOS 24.11)
    environment.sessionVariables = {
      KVANTUM_THEME = "WinSur-dark";
      QT_PLUGIN_PATH = [ "/run/current-system/sw/lib/qt-6/plugins" "/run/current-system/sw/lib/qt-5.15.15/plugins" ];
    };

    system.activationScripts.qt-env = ''
      cat > /etc/environment << 'ENV'
QT_PLUGIN_PATH=/run/current-system/sw/lib/qt-6/plugins:/run/current-system/sw/lib/qt-5.15.15/plugins
KVANTUM_THEME=WinSur-dark
ENV
    '';
{ config, pkgs, lib, ... }:

let
  wallpaperPath = "${pkgs.agntos-branding}/share/wallpapers/agntos/default.png";

  # Custom SDDM theme: Breeze but with our wallpaper
  sddmTheme = pkgs.stdenvNoCC.mkDerivation {
    name = "sddm-breeze-agntos";
    phases = [ "installPhase" ];
    installPhase = ''
      mkdir -p $out/share/sddm/themes/breeze-agntos
      cp -r ${pkgs.kdePackages.plasma-desktop}/share/sddm/themes/breeze/* $out/share/sddm/themes/breeze-agntos/
      chmod +w $out/share/sddm/themes/breeze-agntos/theme.conf
      sed -i "s|^background=.*|background=${wallpaperPath}|" $out/share/sddm/themes/breeze-agntos/theme.conf
      sed -i "s|^type=.*|type=image|" $out/share/sddm/themes/breeze-agntos/theme.conf
    '';
  };
in {
  config = lib.mkIf config.agntos.enable {
    services.displayManager.sddm = {
      enable = true;
      wayland.enable = true;
      theme = "breeze-agntos";
    };
    services.desktopManager.plasma6.enable = true;
    services.xserver.enable = true;

    environment.systemPackages = with pkgs; [
      sddmTheme
      konsole
      dolphin
      plasma-systemmonitor
      papirus-icon-theme
      kora-icon-theme
      agntos-start-icon
      winsur-kde
      # Kvantum for both Qt5 and Qt6 app transparency
      libsForQt5.qtstyleplugin-kvantum
      qt6Packages.qtstyleplugin-kvantum
    ];

    # Enable Kvantum Qt6 style for application transparency
    environment.sessionVariables = {
      KVANTUM_THEME = "WinSur-dark";
    };

    # SDDM reads /etc/environment via PAM — ensure it gets plugin path
    system.activationScripts.qt-env = ''
      cat > /etc/environment << 'ENV'
QT_PLUGIN_PATH=/run/current-system/sw/lib/qt-6/plugins
KVANTUM_THEME=WinSur-dark
ENV
    '';

    programs.dconf.enable = true;
  };
}

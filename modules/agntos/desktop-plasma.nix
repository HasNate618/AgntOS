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
      libsForQt5.qtstyleplugin-kvantum
      qt6Packages.qtstyleplugin-kvantum
      libsForQt5.qt5ct
      qt6Packages.qt6ct
    ];

    # Kvantum via qt5ct platform theme (bypasses KDE theme loading issues)
    environment.sessionVariables = {
      KVANTUM_THEME = "WinSur-dark";
      QT_STYLE_OVERRIDE = "kvantum";
      QT_QPA_PLATFORMTHEME = "qt5ct";
      QT_PLUGIN_PATH = [ "/run/current-system/sw/lib/qt-6/plugins" "/run/current-system/sw/lib/qt-5.15.15/plugins" ];
    };

    # Config files for qt5ct and qt6ct (used when QT_QPA_PLATFORMTHEME=qt5ct)
    system.activationScripts.qt-env = ''
      cat > /etc/environment << 'ENV'
QT_PLUGIN_PATH=/run/current-system/sw/lib/qt-6/plugins:/run/current-system/sw/lib/qt-5.15.15/plugins
QT_QPA_PLATFORMTHEME=qt5ct
QT_STYLE_OVERRIDE=kvantum
KVANTUM_THEME=WinSur-dark
ENV

      mkdir -p /home/developer/.config/qt5ct /home/developer/.config/qt6ct
      cat > /home/developer/.config/qt5ct/qt5ct.conf << 'CFG'
[Appearance]
style=kvantum
color_scheme_path=
icon_theme=agntos-start
CFG
      cp /home/developer/.config/qt5ct/qt5ct.conf /home/developer/.config/qt6ct/qt6ct.conf
      chown -R developer:users /home/developer/.config/qt5ct /home/developer/.config/qt6ct
    '';

    # Kvantum is installed but NOT forced as the active style (known issue)
    # To manually enable: export QT_STYLE_OVERRIDE=kvantum before launching apps
    # Apps use Breeze + WinSurDark color scheme by default

    programs.dconf.enable = true;
  };
}

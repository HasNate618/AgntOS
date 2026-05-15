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
    ];

    # Use NixOS Qt module for proper Qt5/Qt6 theming and plugin paths
    # This sets QT_PLUGIN_PATH, QML2_IMPORT_PATH, QT_QPA_PLATFORMTHEME, and QT_STYLE_OVERRIDE
    # correctly so Kvantum style plugins are discoverable by both Qt5 and Qt6 apps.
    qt = {
      enable = true;
      platformTheme = "qt5ct";
      style = "kvantum";
    };

    # Kvantum theme selection
    environment.sessionVariables = {
      KVANTUM_THEME = "WinSur-dark";
    };

    programs.dconf.enable = true;
  };
}

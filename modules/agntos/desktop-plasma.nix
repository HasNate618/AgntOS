{ config, pkgs, lib, ... }:

let
  wallpaperPath = "${pkgs.agntos-wallpapers}/share/wallpapers/agntos/agntos-bg-1.jpg";

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
      bart-kde
      klassy
    ];

    # Qt plugin path for theme discovery
    environment.sessionVariables = {
      QT_PLUGIN_PATH = [ "/run/current-system/sw/lib/qt-6/plugins" "/run/current-system/sw/lib/qt-5.15.15/plugins" ];
    };

    system.activationScripts.qt-env = ''
      cat > /etc/environment << 'ENV'
QT_PLUGIN_PATH=/run/current-system/sw/lib/qt-6/plugins:/run/current-system/sw/lib/qt-5.15.15/plugins
ENV
    '';

    # Kvantum is installed but NOT forced as the active style (known issue)
    # To manually enable: export QT_STYLE_OVERRIDE=kvantum before launching apps
    # Apps use Breeze + WinSurDark color scheme by default

    programs.dconf.enable = true;
  };
}

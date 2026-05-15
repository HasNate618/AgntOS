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

    # Ensure Kvantum is used as the Qt application style
    environment.sessionVariables = {
      QT_STYLE_OVERRIDE = "kvantum";
    };

    environment.systemPackages = with pkgs; [
      sddmTheme
      konsole
      dolphin
      plasma-systemmonitor
      papirus-icon-theme
      kora-icon-theme
      agntos-start-icon
      winsur-kde
      kdePackages.qtstyleplugin-kvantum
    ];

    programs.dconf.enable = true;
  };
}

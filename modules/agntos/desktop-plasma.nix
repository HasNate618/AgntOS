{ config, pkgs, lib, ... }:

let
  wallpaperPath = "${pkgs.agntos-branding}/share/wallpapers/agntos/default.png";

  # Override Breeze SDDM theme with our wallpaper
  sddmBreezeWithBackground = pkgs.runCommand "sddm-breeze-agntos" {} ''
    mkdir -p $out/share/sddm/themes/breeze
    cp -r ${pkgs.plasma-desktop}/share/sddm/themes/breeze/* $out/share/sddm/themes/breeze/
    sed -i "s|background=.*|background=${wallpaperPath}|" $out/share/sddm/themes/breeze/theme.conf
  '';

in {
  config = lib.mkIf config.agntos.enable {
    services.displayManager.sddm = {
      enable = true;
      wayland.enable = true;
      theme = sddmBreezeWithBackground;
    };
    services.desktopManager.plasma6.enable = true;
    services.xserver.enable = true;

    programs.dconf.enable = true;

    environment.systemPackages = with pkgs; [
      konsole
      dolphin
      plasma-systemmonitor
    ];
  };
}

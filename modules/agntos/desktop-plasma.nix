{ config, pkgs, lib, ... }:

let
  wallpaperPath = "${pkgs.agntos-branding}/share/wallpapers/agntos/default.png";
in {
  config = lib.mkIf config.agntos.enable {
    services.displayManager.sddm = {
      enable = true;
      wayland.enable = true;
      settings.Theme.Background = wallpaperPath;
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

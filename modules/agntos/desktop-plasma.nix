{ config, pkgs, lib, ... }:

{
  config = lib.mkIf config.agntos.enable {
    services.displayManager.sddm = {
      enable = true;
      wayland.enable = true;
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

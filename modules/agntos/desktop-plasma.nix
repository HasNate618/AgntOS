{ config, pkgs, lib, ... }:

{
  config = lib.mkIf config.agntos.enable {
    services.xserver = {
      enable = true;
      displayManager.sddm = {
        enable = true;
        wayland.enable = true;
      };
      desktopManager.plasma6.enable = true;
    };

    # Wayland session
    programs.dconf.enable = true;

    # Useful Plasma packages
    environment.systemPackages = with pkgs; [
      konsole
      dolphin
      plasma-systemmonitor
    ];
  };
}

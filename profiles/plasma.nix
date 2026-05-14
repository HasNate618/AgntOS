{ config, pkgs, lib, ... }:

{
  agntos.enable = true;
  agntos.edition = "home";

  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;

  networking = {
    hostName = "agntos";
    networkmanager.enable = true;
  };

  services.openssh.enable = true;

  system.stateVersion = "24.11";
}

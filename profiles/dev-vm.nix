{ config, pkgs, lib, ... }:

{
  agntos.enable = true;
  agntos.edition = "dev";

  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;

  networking = {
    hostName = "agntos-dev";
    networkmanager.enable = true;
  };

  fileSystems."/" = {
    device = "/dev/disk/by-label/nixos";
    fsType = "ext4";
  };

  users.users.developer = {
    isNormalUser = true;
    initialPassword = "agntos";
    extraGroups = [ "wheel" "networkmanager" ];
  };

  users.users.root.initialPassword = "agntos";

  environment.systemPackages = with pkgs; [
    cargo
    rustc
    rust-analyzer
    clippy
    git
    gcc
    wget
  ];

  programs.bash.interactiveShellInit = ''
    if [ -d /mnt/agntos-src ]; then
      echo "AgntOS source mounted at /mnt/agntos-src"
      alias agntos-rebuild="cd /mnt/agntos-src && cargo build"
    fi
  '';

  system.stateVersion = "24.11";
}

{ config, pkgs, lib, ... }:

{
  boot.kernelParams = [ "console=ttyS0" ];

  virtualisation = {
    qemu = {
      networkingOptions = [ "-nic user,hostfwd=tcp:2222-:22" ];
      memorySize = 8192;
      diskSize = 40960;
      cores = 4;
    };
    graphics = true;
    forwardPorts = [
      { from = "host"; host.port = 2222; guest.port = 22; }
    ];
    sharedDirectories.agntos-source = {
      source = "$PRJ_ROOT";
      target = "/mnt/agntos-src";
    };
  };

  services.openssh.enable = true;

  users.users.root.initialPassword = "agntos";
}

{ config, pkgs, lib, modulesPath, ... }:

{
  imports = [ "${modulesPath}/virtualisation/qemu-vm.nix" ];

  boot.kernelParams = [ "console=ttyS0" ];

  virtualisation = {
    memorySize = 8192;
    diskSize = 40960;
    cores = 4;
    graphics = true;
    qemu.options = let
      src = builtins.getEnv "PRJ_ROOT";
    in lib.optionals (src != "") [
      "-virtfs local,path=${src},mount_tag=agntos-source,security_model=none"
    ];
  };

  networking.firewall.allowedTCPPorts = [ 22 ];
  services.openssh.enable = true;
  users.users.root.initialPassword = "agntos";

  # Mount shared folder if PRJ_ROOT is set
  fileSystems = lib.mkIf (builtins.getEnv "PRJ_ROOT" != "") {
    "/mnt/agntos-src" = {
      device = "agntos-source";
      fsType = "9p";
      options = [ "trans=virtio" "version=9p2000.L" "msize=1048576" ];
    };
  };
}

{ config, pkgs, lib, modulesPath, ... }:

let
  projectRoot = builtins.getEnv "PRJ_ROOT";
  hasSrc = projectRoot != "";
in {

  imports = [ "${modulesPath}/virtualisation/qemu-vm.nix" ];

  boot.kernelParams = [ "console=tty0" "console=ttyS0" ];

  virtualisation = {
    memorySize = 8192;
    diskSize = 40960;
    cores = 4;
    graphics = true;
    forwardPorts = [
      { from = "host"; host.port = 2222; guest.port = 22; }
    ];
    qemu.options = lib.optionals hasSrc [
      "-virtfs local,path=${projectRoot},mount_tag=agntos-source,security_model=none"
    ];
  };

  networking.firewall.allowedTCPPorts = [ 22 ];
  services.openssh.enable = true;
  users.users.root.initialPassword = "agntos";

  boot.initrd.kernelModules = [ "9p" "9pnet_virtio" ];
  systemd.services.agntos-mount = lib.mkIf hasSrc {
    description = "Mount AgntOS source shared folder";
    after = [ "dev-virtio-ports-agntos-source.device" ];
    wants = [ "dev-virtio-ports-agntos-source.device" ];
    wantedBy = [ "multi-user.target" ];
    serviceConfig.Type = "oneshot";
    script = ''
      mkdir -p /mnt/agntos-src
      mount -t 9p -o trans=virtio,version=9p2000.L agntos-source /mnt/agntos-src || true
    '';
    serviceConfig.RemainAfterExit = true;
  };

}

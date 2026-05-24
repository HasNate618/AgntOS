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

  # qemu-vm duplicates overlay options in fstab (x-initrd.mount), which breaks
  # merged /nix/store: paths visible in /nix/.ro-store are invisible to nix-build.
  fileSystems."/nix/store" = lib.mkForce {
    device = "overlay";
    fsType = "overlay";
    options = [
      "lowerdir=/sysroot/nix/.ro-store"
      "upperdir=/sysroot/nix/.rw-store/upper"
      "workdir=/sysroot/nix/.rw-store/work"
    ];
  };

  fileSystems."/nix/.rw-store" = lib.mkForce {
    fsType = "tmpfs";
    options = [
      "mode=0755"
      "size=8G"
    ];
    neededForBoot = true;
  };

  systemd.services.agntos-mount = lib.mkIf hasSrc {
    description = "Mount AgntOS source shared folder";
    after = [ "dev-virtio-ports-agntos-source.device" ];
    wants = [ "dev-virtio-ports-agntos-source.device" ];
    wantedBy = [ "multi-user.target" ];
    serviceConfig = {
      Type = "oneshot";
      RemainAfterExit = true;
    };
    script = ''
      mkdir -p /mnt/agntos-src
      if ${pkgs.util-linux}/bin/mount -t 9p -o trans=virtio,version=9p2000.L agntos-source /mnt/agntos-src; then
        echo "agntos: mounted source at /mnt/agntos-src"
      else
        echo "agntos: source mount failed (rebuild/run VM with PRJ_ROOT set to repo root)" >&2
      fi
    '';
  };

}

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

  users.users.developer = {
    isNormalUser = true;
    initialPassword = "agntos";
    extraGroups = [ "wheel" "networkmanager" ];
  };

  users.users.root.initialPassword = "agntos";

  environment.systemPackages = with pkgs; [
    git
    gcc
    wget
    rustup
  ];

  programs.bash.interactiveShellInit = ''
    if [ -d /mnt/agntos-src ]; then
      alias agntos="cd /mnt/agntos-src"
      alias agnt-build="cd /mnt/agntos-src && cargo build --release"
      alias agnt-check="cd /mnt/agntos-src && cargo check"
      alias agnt-inspect="cd /mnt/agntos-src && cargo run --bin agntctl -- inspect"
      alias agnt-agent="cd /mnt/agntos-src && cargo run --bin agntd"
      alias vbox-mount="sudo mount -t vboxsf agntos-src /mnt/agntos-src"
      export PATH="/mnt/agntos-src/target/release:$PATH"
      echo "AgntOS dev ready: agnt-build | agnt-inspect | agnt-agent"
    fi
  '';

  system.stateVersion = "24.11";
}

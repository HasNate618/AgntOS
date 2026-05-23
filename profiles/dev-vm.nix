{ config, pkgs, lib, ... }:

{
  agntos.enable = true;
  agntos.edition = "dev";
  agntos.agent.enable = true;
  agntos.rebuild.flakeUri = "/mnt/agntos-src#agntos-dev-vm";

  environment.etc."agntos/settings.json".text = builtins.toJSON {
    auto_apply = "auto";
  };

  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;

  networking = {
    hostName = "agntos-dev";
    networkmanager.enable = true;
  };

  users.users.developer = {
    isNormalUser = true;
    initialPassword = "agntos";
    extraGroups = [ "wheel" "networkmanager" "agntos" ];
    openssh.authorizedKeys.keys = [
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB9pAb19Mwl8yl6ZBQbWlDi8eG1AcAMCoN0wOtFvY+wJ nate.e.espejo@gmail.com"
    ];
  };

  security.sudo.extraRules = [
    { groups = [ "wheel" ]; commands = [ { command = "ALL"; options = [ "NOPASSWD" ]; } ]; }
  ];

  users.users.root = {
    initialPassword = "agntos";
    openssh.authorizedKeys.keys = [
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB9pAb19Mwl8yl6ZBQbWlDi8eG1AcAMCoN0wOtFvY+wJ nate.e.espejo@gmail.com"
    ];
  };

  environment.systemPackages = with pkgs; [
    git
    gcc
    wget
    rustup
    agnt
    agntctl
    agntd
  ];

  programs.bash.interactiveShellInit = ''
    if [ -d /mnt/agntos-src ]; then
      alias agntos="cd /mnt/agntos-src"
      alias agnt-build="cd /mnt/agntos-src && cargo build --release"
      alias agnt-check="cd /mnt/agntos-src && cargo check"
      alias agnt-inspect="agnt system inspect"
      alias agnt-agent="agnt"
      export PATH="/mnt/agntos-src/target/release:$PATH"
      echo "AgntOS dev: agnt | agnt system … | agnt-build"
    fi
  '';

  systemd.user.services.agntd.serviceConfig.ExecStart = lib.mkForce
    "/mnt/agntos-src/target/release/agntd --socket /run/agntd/agent.sock";

  system.stateVersion = "24.11";
}

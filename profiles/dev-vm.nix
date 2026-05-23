{ config, pkgs, lib, ... }:

{
  agntos.enable = true;
  agntos.edition = "dev";
  agntos.agent.enable = true;
  agntos.rebuild.flakeUri = "/mnt/agntos-src#agntos-dev-vm";

  environment.etc."agntos/settings.json".text = builtins.toJSON {
    auto_apply = "auto";
  };

  environment.etc."agntos/models.toml".text = ''
    [default]
    endpoint = "http://10.0.0.45/bifrost/v1"
    model = "cohere/command-a-plus-05-2026"
    supports_tools = false

    [routing]
    chat = "default"
    inspect = "default"
    propose = "default"
    memory = "default"
  '';

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
    alias agntos='cd /mnt/agntos-src 2>/dev/null || cd ~'
    alias agnt-inspect='agnt system inspect'
    alias agnt-agent='agnt'
    if [ -x /mnt/agntos-src/target/release/agntctl ]; then
      alias agnt-build='cd /mnt/agntos-src && cargo build --release'
      alias agnt-check='cd /mnt/agntos-src && cargo check'
      export PATH="/mnt/agntos-src/target/release:$PATH"
      echo "AgntOS dev: mounted source at /mnt/agntos-src"
    else
      echo "AgntOS dev: use Nix packages (rebuild VM with PRJ_ROOT set to mount source)"
    fi
  '';

  system.stateVersion = "24.11";
}

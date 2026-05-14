{ config, pkgs, lib, ... }:

{
  options.agntos = {
    enable = lib.mkEnableOption "AgntOS AI-native system extensions";

    edition = lib.mkOption {
      type = lib.types.enum [ "home" "lab" "dev" ];
      default = "home";
      description = "AgntOS edition: home (Plasma), lab (tiling), dev (coding)";
    };

    configDir = lib.mkOption {
      type = lib.types.str;
      default = "/etc/agntos";
      description = "AgntOS-managed Nix config directory";
    };
  };

  config = lib.mkIf config.agntos.enable {
    # Create AgntOS config directory
    systemd.tmpfiles.rules = [
      "d ${config.agntos.configDir} 0755 root root -"
    ];

    environment.systemPackages = with pkgs; [
      nixos-rebuild
    ];
  };
}

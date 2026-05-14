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
    # Create AgntOS config directory tree
    systemd.tmpfiles.rules = [
      "d ${config.agntos.configDir} 0755 root root -"
      "d ${config.agntos.configDir}/proposals 0755 root root -"
      "d ${config.agntos.configDir}/memory 0755 root root -"
      "f ${config.agntos.configDir}/memory/MEMORY.md 0644 root root -"
      "f ${config.agntos.configDir}/memory/USER.md 0644 root root -"
    ];

    environment.etc."agntos/models.toml.example".text = ''
      # AgntOS model routing configuration
      # Copy to /etc/agntos/models.toml and adjust to your endpoint(s).
      # No default endpoint is assumed by AgntOS.

      [default]
      endpoint = "https://api.example.com/v1"
      model = "your-model-name"
      api_key_env = "AGNTOS_API_KEY"
      max_tokens = 4096
      temperature = 0.7

      [routing]
      inspect = "default"
      propose = "default"
      apply = "default"
      chat = "default"
      memory = "default"
    '';

    environment.systemPackages = with pkgs; [
      nixos-rebuild
    ];
  };
}

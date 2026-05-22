{ config, pkgs, lib, ... }:

let
  cfgDir = "/etc/agntos";
  packagesDir = "${cfgDir}/packages";
  optionsDir = "${cfgDir}/options";
  servicesDir = "${cfgDir}/services";

  optionalImport = path: lib.optional (builtins.pathExists path) path;

  dirImports = dir:
    if builtins.pathExists dir then
      let
        entries = builtins.readDir dir;
      in
      map (name: "${dir}/${name}")
      (lib.attrNames (lib.filterAttrs (name: kind:
        (kind == "regular" || kind == "symlink") && lib.hasSuffix ".nix" name
      ) entries))
    else
      [ ];

  packagesImports = dirImports packagesDir;
  optionsImports = dirImports optionsDir;
  serviceImports = dirImports servicesDir;
  homeDir = "${cfgDir}/home";
  homeImports = dirImports homeDir;
in

{
  imports =
    [ ./agntos-cc.nix ]
    ++ packagesImports
    ++ optionalImport "${cfgDir}/custom.nix"
    ++ optionsImports
    ++ serviceImports
    ++ homeImports;

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

    agent = {
      enable = lib.mkEnableOption "AgntOS agent daemon (agntd)";
    };

    rebuild = {
      flakeUri = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = ''
          When set, agntctl apply uses `nixos-rebuild test --flake <uri> --impure`
          instead of the channel-based `nixos-rebuild test`.
          Example: "/home/user/config#my-machine"
        '';
      };
    };
  };

  config = lib.mkIf config.agntos.enable (lib.mkMerge [
    {
      # Create AgntOS config directory tree
      systemd.tmpfiles.rules = [
        "d ${config.agntos.configDir} 0755 root root -"
        "d ${config.agntos.configDir}/packages 0755 root root -"
        "d ${config.agntos.configDir}/options 0755 root root -"
        "d ${config.agntos.configDir}/proposals 0755 root root -"
        "d ${config.agntos.configDir}/services 0755 root root -"
        "d ${config.agntos.configDir}/home 0755 root root -"
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
    }
    (lib.mkIf (config.agntos.rebuild.flakeUri != null) {
      environment.etc."agntos/flake-info".text = config.agntos.rebuild.flakeUri;
    })
  ]);
}

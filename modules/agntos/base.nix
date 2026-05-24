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
    packagesImports
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
      users.groups.agntos = { };

      # Create AgntOS config directory tree
      systemd.tmpfiles.rules = [
        "d ${config.agntos.configDir} 0755 root root -"
        "d ${config.agntos.configDir}/packages 0775 root agntos -"
        "d ${config.agntos.configDir}/options 0775 root agntos -"
        "d ${config.agntos.configDir}/proposals 0775 root agntos -"
        "d ${config.agntos.configDir}/services 0755 root root -"
        "d ${config.agntos.configDir}/home 0755 root root -"
        "d ${config.agntos.configDir}/skills 0755 root root -"
        "d ${config.agntos.configDir}/memory 0775 root agntos -"
        "f ${config.agntos.configDir}/memory/MEMORY.md 0664 root agntos -"
        "f ${config.agntos.configDir}/memory/USER.md 0664 root agntos -"
        "f ${config.agntos.configDir}/models.toml 0664 root agntos -"
      ];

      environment.etc."agntos/models.toml.example".text = ''
        # AgntOS model routing — copy to models.toml and set your endpoints.
        # Pick concrete models in the Control Centre chat dropdown.

        [default]
        endpoint = "http://127.0.0.1:8081/v1"
        model = ""
        api_key_env = "AGNTOS_API_KEY"

        [profiles.gateway]
        endpoint = "http://10.0.0.45/bifrost/v1"
        api_key_env = "AGNTOS_API_KEY"
        model = ""

        [routing]
        chat = "gateway"
        inspect = "gateway"
        propose = "gateway"
        apply = "gateway"
        memory = "gateway"
      '';

      system.activationScripts.agntos-models = lib.stringAfter ["etc"] ''
        if [ ! -s /etc/agntos/models.toml ]; then
          cp /etc/agntos/models.toml.example /etc/agntos/models.toml
        fi
        if [ -w /etc/agntos/models.toml ]; then
          chmod 664 /etc/agntos/models.toml
          chown root:agntos /etc/agntos/models.toml
        fi
      '';

      system.activationScripts.agntos-writable = lib.stringAfter ["etc"] ''
        for d in packages options; do
          if [ -d ${cfgDir}/$d ]; then
            chgrp agntos ${cfgDir}/$d
            chmod 775 ${cfgDir}/$d
          fi
        done
      '';

      environment.etc."agntos/skills".source = ./skills;

      environment.systemPackages = with pkgs; [
        nixos-rebuild
      ];
    }
    (lib.mkIf (config.agntos.rebuild.flakeUri != null) {
      environment.etc."agntos/flake-info".text = config.agntos.rebuild.flakeUri;
    })
  ]);
}

{ config, pkgs, lib, ... }:

let
  cfg = config.agntos.agntos-cc;
in {
  options.agntos.agntos-cc = {
    enable = lib.mkEnableOption "AgntOS Control Centre GUI (agntos-cc)";

    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.agntos-cc;
      description = "The agntos-cc package to use";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = with pkgs; [
      cfg.package
      agntctl
      nodejs
    ];

    environment.etc."agntos/AGENTS.md".source = "${cfg.package}/share/agntos/AGENTS.md";
    environment.etc."agntos/extensions/agntos-tools".source = "${cfg.package}/share/agntos/extensions/agntos-tools";
  };
}

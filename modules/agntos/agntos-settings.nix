{ config, pkgs, lib, ... }:

let
  cfg = config.agntos.settings;
in {
  options.agntos.settings = {
    enable = lib.mkEnableOption "AgntOS Settings GUI";
    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.agntos-settings;
      description = "The agntos-settings package to use";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
  };
}

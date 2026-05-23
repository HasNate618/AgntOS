{ config, pkgs, lib, ... }:

{
  config = lib.mkIf (config.agntos.enable && config.agntos.agent.enable) {
    systemd.user.services.agntd = {
      description = "AgntOS agent daemon";
      wantedBy = [ "default.target" ];
      serviceConfig = {
        Type = "simple";
        ExecStart = "${pkgs.agntd}/bin/agntd --socket %t/agntd.sock";
        Restart = "on-failure";
        RestartSec = 5;
        Environment = [
          "AGNTOS_CONFIG_DIR=${config.agntos.configDir}"
          "AGNTCTL=${pkgs.agntctl}/bin/agntctl"
        ];
        Path = with pkgs; [
          agntctl
          coreutils
          bash
          nix
          gnused
          gnutar
          gzip
        ];
        StandardOutput = "journal";
        StandardError = "journal";
      };
    };
  };
}

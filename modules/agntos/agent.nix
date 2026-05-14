{ config, pkgs, lib, ... }:

{
  config = lib.mkIf config.agntos.agent.enable {
    systemd.user.services.agntd = {
      description = "AgntOS agent daemon";
      wantedBy = [ "default.target" ];
      serviceConfig = {
        Type = "simple";
        ExecStart = "${pkgs.agntd}/bin/agntd";
        Restart = "on-failure";
        RestartSec = 5;
        Environment = "AGNTOS_CONFIG_DIR=${config.agntos.configDir}";
        StandardOutput = "journal";
        StandardError = "journal";
      };
    };
  };
}

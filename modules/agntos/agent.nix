{ config, pkgs, lib, ... }:

{
  config = lib.mkIf config.agntos.agent.enable {
    systemd.user.services.agntd = {
      description = "AgntOS agent daemon";
      wantedBy = [ "default.target" ];
      serviceConfig = {
        Type = "simple";
        ExecStartPre = "${pkgs.coreutils}/bin/mkdir -p /run/agntd";
        ExecStart = "${pkgs.agntd}/bin/agntd --socket /run/agntd/agent.sock";
        Restart = "on-failure";
        RestartSec = 5;
        Environment = "AGNTOS_CONFIG_DIR=${config.agntos.configDir}";
        StandardOutput = "journal";
        StandardError = "journal";
      };
    };
  };
}

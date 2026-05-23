{ config, pkgs, lib, ... }:

in {
  config = lib.mkIf config.agntos.enable {
    services.getty.autologinUser = lib.mkDefault "developer";

    environment.systemPackages = with pkgs; [
      cage
      foot
      tmux
      agnt
      agntctl
      agntd
    ];

    systemd.user.services.agntos-terminal = {
      description = "AgntOS terminal session (Cage + Foot + tmux)";
      wantedBy = [ "graphical-session.target" ];
      after = [ "agntd.service" ];
      serviceConfig = {
        Type = "exec";
        ExecStart = "${pkgs.cage}/bin/cage -s -- ${pkgs.foot}/bin/foot -e ${pkgs.tmux}/bin/tmux new-session -A -s agnt ${pkgs.agnt}/bin/agnt";
        Restart = "on-failure";
        RestartSec = 3;
      };
    };

    systemd.user.targets.graphical-session = {
      unitConfig.DefaultDependencies = false;
      wantedBy = [ "default.target" ];
    };
  };
}

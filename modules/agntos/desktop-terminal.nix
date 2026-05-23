{ config, pkgs, lib, ... }:

let
  agntos-tmux = pkgs.writeShellScriptBin "agntos-tmux" ''
    set -euo pipefail
    CONF=/etc/agntos/dev.tmux.conf
    SESSION=agntos-dev
    SRC=/mnt/agntos-src
    if [ ! -d "$SRC" ]; then
      SRC="$HOME"
      echo "agntos-tmux: /mnt/agntos-src missing, using $SRC" >&2
    fi
    if ${pkgs.tmux}/bin/tmux -f "$CONF" has-session -t "$SESSION" 2>/dev/null; then
      exec ${pkgs.tmux}/bin/tmux -f "$CONF" attach -t "$SESSION"
    fi
    exec ${pkgs.tmux}/bin/tmux -f "$CONF" new-session -s "$SESSION" -n chat -c "$SRC" \; \
      send-keys -t "$SESSION:chat" 'agnt' C-m \; \
      new-window -n shell -c "$SRC" \; \
      new-window -n logs -c "$SRC" \; \
      send-keys -t "$SESSION:logs" 'journalctl --user -u agntd -f' C-m \; \
      select-window -t "$SESSION:chat"
  '';
in {
  config = lib.mkIf config.agntos.enable {
    services.getty.autologinUser = lib.mkDefault "developer";

    environment.etc."agntos/dev.tmux.conf".source = "${./dev-tmux.conf}";

    environment.systemPackages = with pkgs; [
      agntos-tmux
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
        ExecStart = "${pkgs.cage}/bin/cage -s -- ${pkgs.foot}/bin/foot -e ${agntos-tmux}/bin/agntos-tmux";
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

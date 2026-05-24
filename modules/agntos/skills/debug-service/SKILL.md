# Debug a systemd service

1. Use `run_bash` with `systemctl status <unit>` and `journalctl -u <unit> -n 50 --no-pager`.
2. Summarize failure mode (failed, restart loop, missing binary).
3. If a config change is needed, `propose` the change — never edit system files outside `/etc/agntos/`.

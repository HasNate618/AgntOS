{ config, pkgs, lib, ... }:

{
  agntos.enable = true;
  agntos.edition = "dev";
  agntos.agent.enable = true;
  # Inside the VM the source is mounted at /mnt/agntos-src
  agntos.rebuild.flakeUri = "/mnt/agntos-src#agntos-dev-vm";

  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;

  networking = {
    hostName = "agntos-dev";
    networkmanager.enable = true;
  };

  users.users.developer = {
    isNormalUser = true;
    initialPassword = "agntos";
    extraGroups = [ "wheel" "networkmanager" ];
    openssh.authorizedKeys.keys = [
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB9pAb19Mwl8yl6ZBQbWlDi8eG1AcAMCoN0wOtFvY+wJ nate.e.espejo@gmail.com"
    ];
  };

  security.sudo.extraRules = [
    { groups = [ "wheel" ]; commands = [ { command = "ALL"; options = [ "NOPASSWD" ]; } ]; }
  ];

  users.users.root = {
    initialPassword = "agntos";
    openssh.authorizedKeys.keys = [
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB9pAb19Mwl8yl6ZBQbWlDi8eG1AcAMCoN0wOtFvY+wJ nate.e.espejo@gmail.com"
    ];
  };

  environment.systemPackages = with pkgs; [
    git
    gcc
    wget
    rustup
  ];

  programs.bash.interactiveShellInit = ''
    if [ -d /mnt/agntos-src ]; then
      alias agntos="cd /mnt/agntos-src"
      alias agnt-build="cd /mnt/agntos-src && cargo build --release"
      alias agnt-check="cd /mnt/agntos-src && cargo check"
      alias agnt-inspect="cd /mnt/agntos-src && cargo run --bin agntctl -- inspect"
      alias agnt-agent="cd /mnt/agntos-src && cargo run --bin agntd"
      alias agnt-fetch="fastfetch --config /etc/agntos/fastfetch-config.jsonc"
      export PATH="/mnt/agntos-src/target/release:$PATH"
      echo "AgntOS dev ready: agnt-build | agnt-inspect | agnt-fetch"
    fi
  '';

  system.activationScripts.ghostty-config = ''
    # User config directories
    mkdir -p /home/developer/.config/ghostty /home/developer/.config/autostart
    ln -sf /etc/xdg/ghostty/config /home/developer/.config/ghostty/config

    # kdeglobals: WinSur colors + agntos-start icons + AgntOS fonts
    # agntos-start inherits Kora and overrides only the start-menu icon
    cat > /home/developer/.config/kdeglobals << 'KDE'
[General]
ColorScheme=WinSurDark
widgetStyle=kvantum
TerminalApplication=ghostty
[Icons]
Theme=agntos-start
[Fonts]
fixed=GeistMono Nerd Font,10,-1,5,50,0,0,0,0,0
General=Plus Jakarta Sans,10,-1,5,50,0,0,0,0,0
small=Plus Jakarta Sans,8,-1,5,50,0,0,0,0,0
[WM]
activeFont=Plus Jakarta Sans,10,-1,5,50,0,0,0,0,0,Medium
KDE

    # WinSur window decorations + blur
    cat > /home/developer/.config/kwinrc << 'KWI'
[org.kde.kdecoration2]
library=org.kde.kwin.aurorae
theme=__aurorae__svg__WinSur-dark
[Effect-Blur]
Enabled=true
KWI

    # Kvantum theme config
    mkdir -p /home/developer/.config/Kvantum
    cat > /home/developer/.config/Kvantum/kvantum.kvconfig << 'KVN'
[General]
theme=WinSur-dark
KVN

    # Plasma desktop theme
    cat > /home/developer/.config/plasmarc << 'PLA'
[Theme]
name=WinSur-dark
PLA

    # Autostart: apply WinSur theme components individually (preserves icons)
    cat > /home/developer/.config/autostart/agntos-config.desktop << 'DESK'
[Desktop Entry]
Type=Application
Name=AgntOS Config
Exec=/home/developer/.config/autostart/agntos-config.sh
X-KDE-autostart-phase=2
NoDisplay=true
DESK
    cat > /home/developer/.config/autostart/agntos-config.sh << 'SH'
#!/usr/bin/env bash
sleep 3
plasma-apply-desktoptheme WinSur-dark 2>/dev/null &
plasma-apply-colorscheme WinSurDark 2>/dev/null &
qdbus org.kde.KWin /Effects loadEffect "blur" 2>/dev/null &
wait
SH
    chmod +x /home/developer/.config/autostart/agntos-config.sh

    # Splash theme
    printf '[KSplash]\nEngine=KSplashQML\nTheme=agntos-splash\n' > /home/developer/.config/ksplashrc

    # Clear KDE icon cache so it rebuilds with our new icon theme
    rm -f /home/developer/.cache/icon-cache.kcache

    chown -R developer:users /home/developer/.config \
      /home/developer/.local/share /home/developer/.cache 2>/dev/null || true
  '';

  systemd.user.services.agntd.serviceConfig.ExecStart = lib.mkForce
    "/mnt/agntos-src/target/release/agntd --socket /run/agntd/agent.sock";

  system.stateVersion = "24.11";
}

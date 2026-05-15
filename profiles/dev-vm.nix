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
    mkdir -p /home/developer/.config/ghostty /home/developer/.config
    chown developer:users /home/developer/.config
    ln -sf /etc/xdg/ghostty/config /home/developer/.config/ghostty/config

    # Write complete kdeglobals with WinSur dark + Kora icons + AgntOS fonts
    cat > /home/developer/.config/kdeglobals << 'KDE'
[General]
ColorScheme=Bart
widgetStyle=Breeze
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

    # Set Bart window decorations, blur effect, and compositing
    cat > /home/developer/.config/kwinrc << 'KWI'
[org.kde.kdecoration2]
library=org.kde.kwin.aurorae
theme=Bart
[Effect-Blur]
Enabled=true
KWI
    cat > /home/developer/.config/plasmarc << 'PLA'
[Theme]
name=Bart
PLA

    # No Kvantum — uses Breeze widget style with Bart color scheme
    # KWin decorations and blur applied via autostart
    mkdir -p /home/developer/.config/autostart
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
sleep 2
# Apply Bart desktop theme, color scheme, and blur
plasma-apply-desktoptheme Bart 2>/dev/null
plasma-apply-colorscheme Bart 2>/dev/null
qdbus org.kde.KWin /Effects loadEffect "blur" 2>/dev/null
SH
    chmod +x /home/developer/.config/autostart/agntos-config.sh

    # Set splash theme
    printf '[KSplash]\nEngine=KSplashQML\nTheme=agntos-splash\n' > /home/developer/.config/ksplashrc

    chown -R developer:users /home/developer/.config/ghostty /home/developer/.config/kdeglobals \
      /home/developer/.config/ksplashrc /home/developer/.config/kwinrc \
      /home/developer/.config/plasmarc \
      /home/developer/.config/autostart \
      /home/developer/.local/share

    # Customize Bart theme: more transparency, darker colors
    BART_STORE=$(find /nix/store -maxdepth 1 -name "*bart-kde*" -type d | head -1)
    if [ -n "$BART_STORE" ]; then
      # Copy Bart plasma theme to user's writable directory
      mkdir -p /home/developer/.local/share/plasma/desktoptheme/Bart/widgets
      cp -r $BART_STORE/share/plasma/desktoptheme/Bart/* \
        /home/developer/.local/share/plasma/desktoptheme/Bart/
      # Copy color scheme
      mkdir -p /home/developer/.local/share/color-schemes
      cp $BART_STORE/share/color-schemes/Bart.colors \
        /home/developer/.local/share/color-schemes/Bart.colors

      # Patch panel background for more transparency (lower opacity)
      for f in panel-background translucentbackground background; do
        FILE="/home/developer/.local/share/plasma/desktoptheme/Bart/widgets/''${f}.svgz"
        if [ -f "$FILE" ]; then
          zcat < "$FILE" | sed \
            's/stop-opacity:0.49803922/stop-opacity:0.25/g; s/stop-opacity:0.81600001/stop-opacity:0.6/g; s/stop-opacity:0.86000001/stop-opacity:0.5/g; s/stop-opacity:0.875/stop-opacity:0.7/g' \
            | gzip > "''${FILE}.tmp" && mv "''${FILE}.tmp" "$FILE"
        fi
      done

      # Darken Bart color scheme: make backgrounds darker, increase contrast
      sed -i \
        's/BackgroundAlternate=51,48,48/BackgroundAlternate=14,14,16/g; s/BackgroundNormal=51,48,48/BackgroundNormal=18,18,22/g; s/BackgroundAlternate=21,20,20/BackgroundAlternate=14,14,16/g; s/BackgroundNormal=21,20,20/BackgroundNormal=18,18,22/g; s/ForegroundNormal=237,240,242/ForegroundNormal=230,228,220/g; s/ForegroundNormal=201,212,230/ForegroundNormal=210,208,200/g; s/ForegroundNormal=183,186,195/ForegroundNormal=200,198,190/g' \
        /home/developer/.local/share/color-schemes/Bart.colors
      # Add AgntOS orange accent to selection colors
      sed -i \
        's/BackgroundNormal=217,90,60/BackgroundNormal=245,124,72/g; s/BackgroundAlternate=217,90,60/BackgroundAlternate=245,124,72/g' \
        /home/developer/.local/share/color-schemes/Bart.colors
    fi
  '';

  systemd.user.services.agntd.serviceConfig.ExecStart = lib.mkForce
    "/mnt/agntos-src/target/release/agntd --socket /run/agntd/agent.sock";

  system.stateVersion = "24.11";
}

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

  system.activationScripts.plasma-config = ''
    mkdir -p /home/developer/.config
    chown developer:users /home/developer/.config

    # Only write user-facing config files if they don't exist
    # to avoid overwriting user customizations on every rebuild.

    # Write complete kdeglobals with Bart colors + Kora icons + AgntOS fonts
    if [ ! -f /home/developer/.config/kdeglobals ]; then
      cat > /home/developer/.config/kdeglobals << 'KDE'
[General]
ColorScheme=Bart
widgetStyle=Breeze
TerminalApplication=konsole
[Icons]
Theme=agntos-start
[Fonts]
fixed=GeistMono Nerd Font,10,-1,5,50,0,0,0,0,0
General=Plus Jakarta Sans,10,-1,5,50,0,0,0,0,0
small=Plus Jakarta Sans,8,-1,5,50,0,0,0,0,0
[WM]
activeFont=Plus Jakarta Sans,10,-1,5,50,0,0,0,0,0,Medium
KDE
    fi

    # Set Klassy window decorations with rounded corners, blur, compositing (QPainter for VM compat), and animations
    if [ ! -f /home/developer/.config/kwinrc ]; then
      cat > /home/developer/.config/kwinrc << 'KWI'
[org.kde.kdecoration2]
library=org.kde.klassy
theme=klassy
[Compositing]
Enabled=true
Backend=QPainter
[Effect-Blur]
Enabled=true
[Plugins]
blurEnabled=true
fadeEnabled=true
fadedesktopEnabled=true
slidingpopupsEnabled=true
scaleEnabled=true
maximizeEnabled=true
squashEnabled=true
fadingpopupsEnabled=true
KWI
    fi

    # Klassy window decoration config with rounded corners
    mkdir -p /home/developer/.config/klassy
    if [ ! -f /home/developer/.config/klassy/klassyrc ]; then
      cat > /home/developer/.config/klassy/klassyrc << 'KLA'
[Windeco]
WindowCornerRadius=8
RoundAllCornersWhenNoBorders=true
DrawBorderOnMaximizedWindows=true
ColorizeWindowOutlineWithButton=false

[WindowOutlineStyle]
WindowOutlineStyleActive=WindowOutlineCustomColor
WindowOutlineStyleInactive=WindowOutlineCustomWithContrast
WindowOutlineCustomColorActive=245,124,72
WindowOutlineCustomColorInactive=245,124,72
WindowOutlineThickness=1.75
WindowOutlineCustomColorOpacityActive=100
WindowOutlineCustomColorOpacityInactive=60
WindowOutlineCustomWithContrastOpacityActive=80
WindowOutlineCustomWithContrastOpacityInactive=50

[ButtonBehaviour]
ShowOutlineNormallyActive=true
ShowOutlineNormallyInactive=true
ShowCloseOutlineNormallyActive=true
ShowCloseOutlineNormallyInactive=true
KLA
    fi
    if [ ! -f /home/developer/.config/plasmarc ]; then
      cat > /home/developer/.config/plasmarc << 'PLA'
[Theme]
name=Bart
PLA
    fi

    # Create Konsole profile with transparency and blur
    mkdir -p /home/developer/.local/share/konsole
    if [ ! -f /home/developer/.local/share/konsole/AgntOS.profile ]; then
      cat > /home/developer/.local/share/konsole/AgntOS.profile << 'KON'
[General]
Name=AgntOS
Parent=FALLBACK/
Description=AgntOS terminal with transparency

[Appearance]
ColorScheme=Bart
Font=GeistMono Nerd Font,10,-1,5,50,0,0,0,0,0

[Background]
Mode=Blur
KON
    fi

    # System-managed configs always get written on rebuild
    cat > /home/developer/.config/konsolerc << 'KRC'
[Desktop Entry]
DefaultProfile=AgntOS.profile
KRC

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
# Load built-in animation effects
for e in fade fadedesktop slidingpopups scale maximize squash fadingpopups; do
  qdbus org.kde.KWin /Effects loadEffect "$e" 2>/dev/null
done
SH
    chmod +x /home/developer/.config/autostart/agntos-config.sh

    # Set splash theme
    if [ ! -f /home/developer/.config/ksplashrc ]; then
      printf '[KSplash]\nEngine=KSplashQML\nTheme=agntos-splash\n' > /home/developer/.config/ksplashrc
    fi

    # Clear stale cache — always runs to stay in sync with config changes
    rm -f /home/developer/.cache/plasma_theme_Bart.kcache
    rm -f /home/developer/.cache/icon-cache.kcache

    chown -R developer:users /home/developer/.config/kdeglobals \
      /home/developer/.config/ksplashrc /home/developer/.config/kwinrc \
      /home/developer/.config/plasmarc /home/developer/.config/konsolerc \
      /home/developer/.config/klassy \
      /home/developer/.config/autostart \
      /home/developer/.local/share/konsole \
      /home/developer/.local/share \
      /home/developer/.cache
  '';

  systemd.user.services.agntd.serviceConfig.ExecStart = lib.mkForce
    "/mnt/agntos-src/target/release/agntd --socket /run/agntd/agent.sock";

  system.stateVersion = "24.11";
}

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
      export QT_STYLE_OVERRIDE=kvantum
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
ColorScheme=WinSurDark
widgetStyle=kvantum
TerminalApplication=ghostty
[Icons]
Theme=kora
[KDE]
LookAndFeelPackage=com.github.yeyushengfan258.WinSur-dark
[Fonts]
fixed=GeistMono Nerd Font,10,-1,5,50,0,0,0,0,0
General=Plus Jakarta Sans,10,-1,5,50,0,0,0,0,0
small=Plus Jakarta Sans,8,-1,5,50,0,0,0,0,0
[WM]
activeFont=Plus Jakarta Sans,10,-1,5,50,0,0,0,0,0,Medium
KDE

    # Set WinSur window decorations, blur effect, and compositing
    cat > /home/developer/.config/kwinrc << 'KWI'
[org.kde.kdecoration2]
library=org.kde.kwin.aurorae
theme=__aurorae__svg__WinSur-dark
[Effect-Blur]
Enabled=true
KWI
    cat > /home/developer/.config/plasmarc << 'PLA'
[Theme]
name=WinSur-dark
PLA

    # Set Kvantum theme for transparency
    mkdir -p /home/developer/.config/Kvantum
    cat > /home/developer/.config/Kvantum/kvantum.kvconfig << 'KVN'
[General]
theme=WinSur-dark
KVN

    # Override start button icon with AgntOS logo (XDG hicolor path)
    mkdir -p /home/developer/.local/share/icons/hicolor/64x64/places
    cp ${pkgs.agntos-branding}/share/agntos/logos/agntos.svg \
      /home/developer/.local/share/icons/hicolor/64x64/places/start-here-kde-plasma.svg

    # Set splash theme
    printf '[KSplash]\nEngine=KSplashQML\nTheme=agntos-splash\n' > /home/developer/.config/ksplashrc

    chown -R developer:users /home/developer/.config/ghostty /home/developer/.config/kdeglobals \
      /home/developer/.config/ksplashrc /home/developer/.config/kwinrc \
      /home/developer/.config/plasmarc /home/developer/.config/Kvantum \
      /home/developer/.local/share
  '';

  systemd.user.services.agntd.serviceConfig.ExecStart = lib.mkForce
    "/mnt/agntos-src/target/release/agntd --socket /run/agntd/agent.sock";

  system.stateVersion = "24.11";
}

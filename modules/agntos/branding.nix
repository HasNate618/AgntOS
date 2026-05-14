{ config, pkgs, lib, ... }:

let
  wallpaperPath = "${pkgs.agntos-branding}/share/wallpapers/agntos/default.png";
in {

  # ── Distro identity ──

  environment.etc = {
    "os-release".text = lib.mkForce ''
      ANSI_COLOR="38;2;245;124;72"
      BUILD_ID=${config.system.nixos.revision or "unknown"}
      HOME_URL="https://agntos.ai"
      ID=agntos
      ID_LIKE=nixos
      LOGO=agntos
      NAME="AgntOS"
      PRETTY_NAME="AgntOS 24.11 (Vicuna)"
      VERSION="24.11 (Vicuna)"
      VERSION_CODENAME=vicuna
      VERSION_ID=24.11
    '';

    "agntos/logo.txt".source = "${pkgs.agntos-branding}/share/agntos/logo.txt";
    "agntos/fastfetch-config.jsonc".source =
      "${pkgs.agntos-branding}/share/agntos/fastfetch-config.jsonc";

    # Make it the system-wide default so `fastfetch` uses it automatically
    "fastfetch/config.jsonc".source =
      "${pkgs.agntos-branding}/share/agntos/fastfetch-config.jsonc";
  };

  # ── Hostname ──

  networking.hostName = lib.mkDefault "agntos";

  # ── Wallpaper (new users) ──
  # Set system-wide default for new Plasma users
  environment.etc."xdg/plasma-org.kde.plasma.desktop-appletsrc".text = ''
    [Containments][2][Wallpaper][org.kde.image][General]
    Image=file://${wallpaperPath}
    FillMode=2
  '';

  # ── Wallpaper (existing users) ──
  # Runs on every Plasma login via autostart
  environment.etc."xdg/autostart/agntos-wallpaper.desktop".text = ''
    [Desktop Entry]
    Type=Application
    Name=AgntOS Wallpaper
    Exec=plasma-apply-wallpaperimage ${wallpaperPath}
    X-KDE-autostart-phase=2
    NoDisplay=true
  '';

  # ── KDE Plasma defaults ──

  environment.etc."xdg/kdeglobals".text = ''
    [General]
    ColorScheme=BreezeDark
    widgetStyle=Breeze
    TerminalApplication=ghostty
    [KDE]
    LookAndFeelPackage=org.kde.breezedark.desktop
    [Icons]
    Theme=Papirus-Dark
    [Fonts]
    fixed=GeistMono Nerd Font,10,-1,5,50,0,0,0,0,0
    General=Plus Jakarta Sans,10,-1,5,50,0,0,0,0,0
    small=Plus Jakarta Sans,8,-1,5,50,0,0,0,0,0
    [WM]
    activeFont=Plus Jakarta Sans,10,-1,5,50,0,0,0,0,0,Medium
  '';

  environment.etc."xdg/kwinrc".text = ''
    [org.kde.kdecoration2]
    BorderSize=Normal
    BorderlessMaximizedWindows=true
    Theme=__aurorae__svg__kde-rich
  '';

  environment.etc."xdg/plasmarc".text = ''
    [Theme]
    name=breeze-dark
    useCustomStyle=true
  '';

  # Lock screen wallpaper
  environment.etc."xdg/kscreenlockerrc".text = ''
    [Greeter][Wallpaper][org.kde.image][General]
    Image=file://${wallpaperPath}
    FillMode=2
  '';

  # ── Fonts ──

  fonts.packages = with pkgs; [
    (nerdfonts.override { fonts = [ "GeistMono" ]; })
    agntos-fonts
  ];

  fonts.fontconfig.defaultFonts = {
    sansSerif = [ "Plus Jakarta Sans" ];
    monospace = [ "GeistMono Nerd Font" ];
  };

  # ── Ghostty terminal ──

  environment.etc."xdg/ghostty/config".text = ''
    # AgntOS default terminal configuration
    font-family = GeistMono Nerd Font
    font-size = 11
    font-feature = calt
    font-feature = liga

    # AgntOS color palette
    background = #141416
    foreground = #e0e0e0
    cursor-color = #F57C48
    selection-background = #F57C48
    selection-foreground = #141416

    palette = 0=#141416
    palette = 1=#e06c75
    palette = 2=#98c379
    palette = 3=#d19a66
    palette = 4=#61afef
    palette = 5=#c678dd
    palette = 6=#56b6c2
    palette = 7=#abb2bf
    palette = 8=#5c6370
    palette = 9=#e06c75
    palette = 10=#98c379
    palette = 11=#d19a66
    palette = 12=#61afef
    palette = 13=#c678dd
    palette = 14=#56b6c2
    palette = 15=#ffffff

    cursor-style = bar
    cursor-style-blink = true
    background-opacity = 0.95
    window-padding-x = 8
    window-padding-y = 8
  '';

  # ── System packages ──

  environment.systemPackages = with pkgs; [
    agntos-branding
    fastfetch
    ghostty
  ];

}

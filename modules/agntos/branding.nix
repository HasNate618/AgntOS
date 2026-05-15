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

  environment.sessionVariables = {
    GHOSTTY_CONFIG = "/etc/xdg/ghostty/config";
  };

  environment.etc."xdg/ghostty/config".text = ''
    # AgntOS default terminal configuration
    font-family = GeistMono Nerd Font
    font-size = 11
    font-feature = calt
    font-feature = liga

    # AgntOS color palette — warm dark theme
    background = #141416
    foreground = #E2E0D8
    cursor-color = #F57C48
    cursor-style = bar
    cursor-style-blink = true
    selection-background = #F57C48
    selection-foreground = #141416

    palette = 0=#1E1E22
    palette = 1=#E06B6B
    palette = 2=#7BC97A
    palette = 3=#E5B567
    palette = 4=#6CB6EB
    palette = 5=#C77DBB
    palette = 6=#5EBFC5
    palette = 7=#D4D2CC
    palette = 8=#3E3E44
    palette = 9=#FF7A7A
    palette = 10=#8ED98D
    palette = 11=#F0C870
    palette = 12=#7EC4F0
    palette = 13=#D48AC8
    palette = 14=#6ECCD2
    palette = 15=#F2F0EA

    background-opacity = 0.95
    window-padding-x = 8
    window-padding-y = 8
  '';

  # ── Boot splash (Plymouth) ──

  boot.plymouth = {
    enable = true;
    theme = "spinner";
    logo = "${pkgs.agntos-branding}/share/agntos/logos/agntos.png";
  };

  boot.initrd.kernelModules = [ "bochs" ];

  # ── System packages ──

  environment.systemPackages = with pkgs; [
    agntos-branding
    fastfetch
    ghostty
  ];

}

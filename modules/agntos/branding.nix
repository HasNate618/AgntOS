{ config, pkgs, lib, ... }:

let
  wallpaperPath = "${pkgs.agntos-wallpapers}/share/wallpapers/agntos/agntos-bg-1.jpg";
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

  system.nixos.distroName = lib.mkDefault "AgntOS";
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

  environment.etc."xdg/ksplashrc".text = ''
    [KSplash]
    Engine=KSplashQML
    Theme=agntos-splash
  '';

  environment.etc."xdg/kdeglobals".text = ''
    [General]
    ColorScheme=BreezeDark
    widgetStyle=Breeze
    TerminalApplication=konsole
    [KDE]
    LookAndFeelPackage=org.kde.breezedark.desktop
    [Icons]
    Theme=agntos-start
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
    library=org.kde.klassy
    theme=klassy
    [Plugins]
    fadeEnabled=true
    fadedesktopEnabled=true
    slidingpopupsEnabled=true
    scaleEnabled=true
    maximizeEnabled=true
    squashEnabled=true
    fadingpopupsEnabled=true
  '';

  environment.etc."xdg/klassy/klassyrc".text = ''
    [Windeco]
    WindowCornerRadius=8
    RoundAllCornersWhenNoBorders=true
    ThinWindowOutlineStyleActive=WindowOutlineCustomColor
    ThinWindowOutlineStyleInactive=WindowOutlineCustomWithContrast
    ThinWindowOutlineCustomColorActive=245,124,72
    ThinWindowOutlineCustomColorInactive=245,124,72
    ThinWindowOutlineThickness=1.75
    WindowOutlineCustomColorOpacityActive=100
    WindowOutlineCustomColorOpacityInactive=60
    WindowOutlineCustomWithContrastOpacityActive=80
    WindowOutlineCustomWithContrastOpacityInactive=50
    ShowOutlineNormallyActive=true
    ShowOutlineNormallyInactive=true
    ShowCloseOutlineNormallyActive=true
    ShowCloseOutlineNormallyInactive=true
    DrawBorderOnMaximizedWindows=true
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

  # ── Boot splash (Plymouth) ──

  boot.plymouth = {
    enable = true;
    theme = "breeze";
    logo = "${pkgs.agntos-branding}/share/agntos/logos/agntos.png";
  };

  boot.initrd = {
    kernelModules = [ "bochs" "drm" ];
    systemd.enable = true;
    verbose = false;
  };

  # Silent boot — suppress kernel messages for clean Plymouth display
  boot.consoleLogLevel = 3;
  boot.kernelParams = [
    "quiet"
    "udev.log_level=3"
    "systemd.show_status=auto"
  ];
  boot.loader.timeout = 0;

  # ── System packages ──

  environment.systemPackages = with pkgs; [
    agntos-branding
    agntos-wallpapers
    fastfetch
  ];

}

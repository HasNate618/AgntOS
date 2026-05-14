{ config, pkgs, lib, ... }:

let
  agntosBranding = pkgs.callPackage ../../pkgs/agntos-branding { };
  wallpaperPath = "${agntosBranding}/share/wallpapers/agntos/default.png";
in {

  # ── Distro identity ──

  environment.etc = {
    "os-release".text = ''
      PRETTY_NAME="AgntOS 24.11 (Vicuna)"
      NAME="AgntOS"
      ID=agntos
      ID_LIKE=nixos
      VERSION_ID=24.11
      VERSION="24.11 (Vicuna)"
      VERSION_CODENAME=vicuna
      BUILD_ID=${config.system.nixos.revision or "unknown"}
      HOME_URL="https://agntos.ai"
      LOGO=agntos
      ANSI_COLOR="38;2;245;124;72"
    '';

    "agntos/logo.txt".source = "${agntosBranding}/share/agntos/logo.txt";
    "agntos/fastfetch-config.jsonc".source =
      "${agntosBranding}/share/agntos/fastfetch-config.jsonc";

    # Make it the system-wide default so `fastfetch` uses it automatically
    "fastfetch/config.jsonc".source =
      "${agntosBranding}/share/agntos/fastfetch-config.jsonc";
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
  ];

  fonts.fontconfig.defaultFonts = {
    sansSerif = [ "Plus Jakarta Sans" ];
    monospace = [ "GeistMono Nerd Font" ];
  };

  # ── System packages ──

  environment.systemPackages = with pkgs; [
    agntosBranding
    fastfetch
  ];

}

{ config, pkgs, lib, ... }:

let
  agntosBranding = pkgs.callPackage ../../pkgs/agntos-branding { };
in {

  # ── Distro identity (override /etc/os-release) ──

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
      DOCUMENTATION_URL="https://agntos.ai/docs"
      LOGO=agntos
      ANSI_COLOR="38;2;245;124;72"
    '';

    "agntos/logo.txt".source = "${agntosBranding}/share/agntos/logo.txt";

    "agntos/fastfetch-config.jsonc".source =
      "${agntosBranding}/share/agntos/fastfetch-config.jsonc";
  };

  # ── Hostname ──

  networking.hostName = lib.mkDefault "agntos";

  # ── Boot loader branding ──

  boot.loader.systemd-boot.editor = false;
  boot.loader.efi.canTouchEfiVariables = true;

  # ── Wallpapers + Fastfetch ──

  environment.systemPackages = with pkgs; [
    agntosBranding
    fastfetch
  ];

  # Set the default Plasma wallpaper to AgntOS default
  environment.etc."xdg/plasma-org.kde.plasma.desktop-appletsrc".text = ''
    [Containments][2][Wallpaper][org.kde.image][General]
    Image=file://${agntosBranding}/share/wallpapers/agntos/default.png
    FillMode=2
  '';

  # Apply wallpaper on login for existing users
  programs.bash.loginShellInit = ''
    if command -v plasma-apply-wallpaperimage &>/dev/null; then
      plasma-apply-wallpaperimage ${agntosBranding}/share/wallpapers/agntos/default.png &>/dev/null || true
    fi
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
    smallestReadableFont=Plus Jakarta Sans,8,-1,5,50,0,0,0,0,0

    [WM]
    activeFont=Plus Jakarta Sans,10,-1,5,50,0,0,0,0,0,Medium
  '';

  environment.etc."xdg/kwinrc".text = ''
    [org.kde.kdecoration2]
    BorderSize=Normal
    BorderlessMaximizedWindows=true
    CloseOnDoubleClickOnMenu=false
    Library=org.kde.kwin.aurorae
    Theme=__aurorae__svg__kde-rich
  '';

  environment.etc."xdg/plasmarc".text = ''
    [Theme]
    name=breeze-dark
    useCustomStyle=true
  '';

  # ── Fonts ──

  fonts.packages = with pkgs; [
    (nerdfonts.override { fonts = [ "GeistMono" ]; })
  ];

  fonts.fontconfig.defaultFonts = {
    sansSerif = [ "Plus Jakarta Sans" ];
    monospace = [ "GeistMono Nerd Font" ];
  };

  # ── Fastfetch shell aliases ──

  environment.shellAliases = {
    neofetch = "fastfetch --config /etc/agntos/fastfetch-config.jsonc";
    ff = "fastfetch --config /etc/agntos/fastfetch-config.jsonc";
  };

  # ── Desktop theme helpers ──

}

{
  description = "AgntOS - AI-native Linux distribution";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
  };

  outputs = { self, nixpkgs, ... }: let
    agntosOverlay = final: prev: {
      agntctl = final.callPackage ./pkgs/agntctl { };
      agntd = final.callPackage ./pkgs/agntd { };
      agntos-branding = final.callPackage ./pkgs/agntos-branding { };
      agntos-fonts = final.callPackage ./pkgs/agntos-fonts { };
      agntos-start-icon = final.callPackage ./pkgs/agntos-start-icon { };
      agntos-wallpapers = final.callPackage ./pkgs/agntos-wallpapers { };
      bart-kde = final.callPackage ./pkgs/bart-kde { };
      winsur-kde = final.callPackage ./pkgs/winsur-kde { };
    };
  in {
    # --- Dev VM ---
    nixosConfigurations.agntos-dev-vm = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        ./modules/agntos/base.nix
        ./modules/agntos/agent.nix
        ./modules/agntos/branding.nix
        ./modules/agntos/desktop-plasma.nix
        ./modules/agntos/vm.nix
        ./profiles/dev-vm.nix
        ({ ... }: { nixpkgs.overlays = [ agntosOverlay ]; })
      ];
    };

    # --- Plasma-only system ---
    nixosConfigurations.agntos-plasma = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        ./modules/agntos/base.nix
        ./modules/agntos/agent.nix
        ./modules/agntos/branding.nix
        ./modules/agntos/desktop-plasma.nix
        ./profiles/plasma.nix
        ({ ... }: { nixpkgs.overlays = [ agntosOverlay ]; })
      ];
    };

    # --- Packages ---
    packages.x86_64-linux = let
      pkgs = import nixpkgs {
        system = "x86_64-linux";
        overlays = [ agntosOverlay ];
      };
    in {
      inherit (pkgs) agntctl agntd agntos-branding agntos-fonts agntos-start-icon agntos-wallpapers bart-kde winsur-kde;
    };
  };
}

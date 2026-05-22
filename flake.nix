{
  description = "AgntOS - AI-native Linux distribution";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs, nixpkgs-unstable, ... }: let
    agntosOverlay = final: prev: {
      agntctl = final.callPackage ./pkgs/agntctl {
        rustPlatform = nixpkgs-unstable.legacyPackages.${final.stdenv.hostPlatform.system}.rustPlatform;
      };
      agntd = final.callPackage ./pkgs/agntd {
        rustPlatform = nixpkgs-unstable.legacyPackages.${final.stdenv.hostPlatform.system}.rustPlatform;
      };
      agntos-branding = final.callPackage ./pkgs/agntos-branding { };
      agntos-fonts = final.callPackage ./pkgs/agntos-fonts { };
      agntos-cc-frontend-src = final.callPackage ./pkgs/agntos-cc/frontend-src.nix { };
      agntos-cc-frontend = final.callPackage ./pkgs/agntos-cc/frontend.nix {
        agntos-cc-frontend-src = final.agntos-cc-frontend-src;
      };
      agntos-cc = final.callPackage ./pkgs/agntos-cc {
        rustPlatform = nixpkgs-unstable.legacyPackages.${final.stdenv.hostPlatform.system}.rustPlatform;
        agntos-cc-frontend = final.agntos-cc-frontend;
      };
      pi-coding-agent = final.callPackage ./pkgs/pi-coding-agent { };
      agntos-start-icon = final.callPackage ./pkgs/agntos-start-icon { };
      agntos-wallpapers = final.callPackage ./pkgs/agntos-wallpapers { };
      bart-kde = final.callPackage ./pkgs/bart-kde { };
      winsur-kde = final.callPackage ./pkgs/winsur-kde { };
      kirigami = final.kdePackages.kirigami;
      klassy = nixpkgs-unstable.legacyPackages.${final.stdenv.hostPlatform.system}.klassy;
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
      inherit (pkgs) agntctl agntd agntos-branding agntos-cc agntos-cc-frontend agntos-cc-frontend-src agntos-fonts agntos-start-icon agntos-wallpapers bart-kde pi-coding-agent winsur-kde klassy;
    };
  };
}

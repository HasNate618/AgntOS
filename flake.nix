{
  description = "AgntOS - AI-native Linux distribution";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
  };

  outputs = { self, nixpkgs, ... }: let
    pkgsFor = system: import nixpkgs { inherit system; };
  in {
    # --- Dev VM ---
    nixosConfigurations.agntos-dev-vm = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        ./modules/agntos/base.nix
        ./modules/agntos/desktop-plasma.nix
        ./modules/agntos/vm.nix
        ./profiles/dev-vm.nix
      ];
    };

    # --- Plasma-only system ---
    nixosConfigurations.agntos-plasma = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        ./modules/agntos/base.nix
        ./modules/agntos/desktop-plasma.nix
        ./profiles/plasma.nix
      ];
    };

    # --- Packages ---
    packages.x86_64-linux = let
      pkgs = pkgsFor "x86_64-linux";
    in {
      agntctl = pkgs.callPackage ./pkgs/agntctl { };
      agntd = pkgs.callPackage ./pkgs/agntd { };
    };
  };
}

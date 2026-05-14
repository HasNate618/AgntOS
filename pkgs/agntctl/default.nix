{ lib, rustPlatform, pkg-config, openssl }:

rustPlatform.buildRustPackage {
  pname = "agntctl";
  version = "0.1.0";

  src = lib.cleanSource ../..;

  cargoBuildFlags = [ "-p" "agntctl" ];

  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ openssl ];

  cargoLock = {
    lockFile = ../../Cargo.lock;
  };

  meta = {
    description = "AgntOS control tool for Nix-backed system changes";
    license = lib.licenses.mit;
    mainProgram = "agntctl";
  };
}
{ lib, rustPlatform, pkg-config, openssl, dbus }:

rustPlatform.buildRustPackage {
  pname = "agntctl";
  version = "0.1.0";

  src = ../../crates/agntctl;

  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ openssl dbus ];

  cargoLock = {
    lockFile = ../../Cargo.lock;
  };

  meta = {
    description = "AgntOS control tool for Nix-backed system changes";
    license = lib.licenses.mit;
    mainProgram = "agntctl";
  };
}

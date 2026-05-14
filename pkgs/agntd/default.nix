{ lib, rustPlatform, pkg-config, openssl, dbus }:

rustPlatform.buildRustPackage {
  pname = "agntd";
  version = "0.1.0";

  src = ../../crates/agntd;

  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ openssl dbus ];

  cargoLock = {
    lockFile = ../../Cargo.lock;
  };

  meta = {
    description = "AgntOS agent daemon";
    license = lib.licenses.mit;
    mainProgram = "agntd";
  };
}

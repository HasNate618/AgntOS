{ lib, rustPlatform, pkg-config, openssl }:

rustPlatform.buildRustPackage {
  pname = "agntd";
  version = "0.1.0";

  src = lib.cleanSource ../..;

  cargoBuildFlags = [ "-p" "agntd" ];

  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ openssl ];

  cargoLock = {
    lockFile = ../../Cargo.lock;
  };

  meta = {
    description = "AgntOS agent daemon";
    license = lib.licenses.mit;
    mainProgram = "agntd";
  };
}
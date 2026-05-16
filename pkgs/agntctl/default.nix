{ lib, rustPlatform, pkg-config, openssl }:

rustPlatform.buildRustPackage {
  pname = "agntctl";
  version = "0.1.0";

  src = lib.cleanSourceWith {
    filter = path: type:
      lib.cleanSourceFilter path type
      && !lib.hasSuffix ".png" (baseNameOf path)
      && !lib.hasSuffix ".jpg" (baseNameOf path)
      && !lib.hasSuffix ".jpeg" (baseNameOf path)
      && !lib.hasSuffix ".svg" (baseNameOf path)
      && !lib.hasSuffix ".qcow2" (baseNameOf path);
    src = ../..;
  };

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
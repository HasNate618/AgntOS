{ lib, rustPlatform }:

rustPlatform.buildRustPackage {
  pname = "agnt";
  version = "0.1.0";

  src = lib.cleanSourceWith {
    filter = path: type:
      lib.cleanSourceFilter path type
      && !lib.hasSuffix ".qcow2" (baseNameOf path);
    src = ../..;
  };

  cargoBuildFlags = [ "-p" "agnt" ];

  postPatch = ''
    substituteInPlace Cargo.toml \
      --replace-fail $'members = [\n  "crates/agnt-common",\n  "crates/agntctl",\n  "crates/agntd",\n]' \
      $'members = ["crates/agnt"]\ndefault-members = ["crates/agnt"]'
  '';

  cargoLock = {
    lockFile = ../../Cargo.lock;
  };

  meta = {
    description = "AgntOS unified CLI";
    license = lib.licenses.gpl3Plus;
    mainProgram = "agnt";
  };
}

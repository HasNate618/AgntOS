{ lib, buildNpmPackage, agntos-cc-frontend-src }:

buildNpmPackage rec {
  pname = "agntos-cc-frontend";
  version = "0.1.0";

  src = agntos-cc-frontend-src;

  npmDepsHash = "sha256-taZ/AymSmuDSuEtIDRMHX/WJFHDNO7xgL2rk+IJy1NI=";

  npmFlags = [ "--include=dev" ];

  installPhase = ''
    runHook preInstall
    mkdir -p $out
    cp -r dist $out/
    runHook postInstall
  '';

  meta = {
    description = "Built web assets for AgntOS Control Centre";
    license = lib.licenses.mit;
  };
}

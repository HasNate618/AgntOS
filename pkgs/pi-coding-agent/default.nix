{ lib, fetchurl, buildNpmPackage, nodejs }:

buildNpmPackage rec {
  pname = "pi-coding-agent";
  version = "0.75.3";

  src = fetchurl {
    url = "https://registry.npmjs.org/@earendil-works/pi-coding-agent/-/pi-coding-agent-${version}.tgz";
    hash = "sha256-aZLAoy8BhRJuJVHsrK54K2It75QiAhui5+91OBt0Fow=";
  };

  npmDepsHash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

  dontNpmBuild = true;

  meta = {
    description = "Pi coding agent - hidden runtime dependency for agntos-cc";
    longDescription = ''
      Pi is an LLM-powered coding and system administration agent.
      It is used as a hidden runtime dependency by agntos-cc, which
      spawns Pi in RPC mode and communicates via stdin/stdout.
    '';
    homepage = "https://github.com/earendil-works/pi-coding-agent";
    license = lib.licenses.mit;
    platforms = lib.platforms.linux;
  };
}

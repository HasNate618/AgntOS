{ lib, nodejs, stdenv, writeShellScriptBin }:

let
  version = "0.75.3";
  package = "@earendil-works/pi-coding-agent@${version}";
  npmPrefix = stdenv.mkDerivation {
    pname = "pi-coding-agent-npm";
    version = version;
    nativeBuildInputs = [ nodejs ];
    dontUnpack = true;
    buildPhase = ''
      export HOME="$TMPDIR"
      export npm_config_cache="$TMPDIR/npm-cache"
      mkdir -p "$out/lib/node_modules"
      cd "$out/lib/node_modules"
      ${nodejs}/bin/npm install --omit=dev --no-audit --no-fund ${package}
    '';
    installPhase = ''
      runHook preInstall
      runHook postInstall
    '';
  };
in
writeShellScriptBin "pi" ''
  export PATH="${lib.makeBinPath [ nodejs ]}:$PATH"
  exec ${nodejs}/bin/node \
    "${npmPrefix}/lib/node_modules/@earendil-works/pi-coding-agent/dist/cli.js" \
    "$@"
''

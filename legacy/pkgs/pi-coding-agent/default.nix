{ lib, nodejs, writeShellScriptBin }:

let
  version = "0.75.3";
  package = "@earendil-works/pi-coding-agent@${version}";
in
writeShellScriptBin "pi" ''
  export PATH="${lib.makeBinPath [ nodejs ]}:$PATH"
  exec ${nodejs}/bin/npx --yes ${package} "$@"
''

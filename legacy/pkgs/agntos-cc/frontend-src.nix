{ runCommand }:

runCommand "agntos-cc-frontend-src" {} ''
  mkdir -p $out
  cp -a ${../../agntos-cc/frontend}/. $out/
  chmod -R u+w $out
  rm -rf $out/node_modules $out/dist
  test -f $out/src/main.tsx
''

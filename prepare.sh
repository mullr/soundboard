#!/usr/bin/env bash
set -e
VERSION="10.10.6"

rm preact-*tgz 
curl -OL https://github.com/preactjs/preact/releases/download/${VERSION}/preact-${VERSION}.tgz
tar xf preact-10.10.6.tgz 

# put out ES6 module tire fire
for file in $(find package -name "*.mjs"); do
    sed -i 's/from"preact"/from".\/preact.mjs"/' $file
    sed -i 's/from"preact\/hooks"/from".\/hooks.mjs"/' $file
    sed -i 's/import"preact\/devtools"/import".\/devtools.mjs"/' $file
done

rm -rf src/public/preact
mv package/ src/public/preact

#!/bin/sh

echo "Updating js/api-augment"
cd js/api-augment
rm package-lock.json
npx npm-check-updates -u
npm i
npm run build
cd dist
npm pack
cd ../../..

echo "Updating js/schemas"
cd js/schemas
rm package-lock.json
npx npm-check-updates -u
npm i
cd ../..

echo "Updating js/recovery-sdk"
cd js/recovery-sdk
rm package-lock.json
npx npm-check-updates -u
npm i
cd ../..

echo "Updating js/ethereum-utils"
cd js/ethereum-utils
rm package-lock.json
npx npm-check-updates -u
npm i
npm run build
cd dist
npm pack
cd ../../..

echo "Updating e2e"
cd e2e
rm package-lock.json
npx npm-check-updates -u
npm i ../js/api-augment/dist/frequency-chain-api-augment-0.0.0.tgz
cd ..

echo "Updating tools/state-copy"
cd tools/state-copy
rm package-lock.json
npx npm-check-updates -u
npm i
cd ../..

echo "Updating tools/genesis-data"
cd tools/genesis-data
rm package-lock.json
npx npm-check-updates -u
npm i
cd ../..

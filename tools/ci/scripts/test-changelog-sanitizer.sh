#!/bin/bash
# This script is used for testing string substitutions which happen in
# "Sanitize Changelog" of the Release workflow.

set -x
changelog='### Major Changes|n|

  - Restrict Pays:No call in batches #1452|n|
  - refactor: feature frequency-local => frequency-paseo-local #1511|n|

  |n||n|### Uncategorized Changes|n||n| - [Handles] Testing Cleanup #1445|n|
  - fix type definition mismatch for schemas #1443|n|
  - fix git related warning #1444|n|
  - Increase test coverage and Minor RPC Bug #1448|n|
  - Bug: Fix missing extension warning in api-augment #1457|n|
  - pin polkadotjs api to running version #1462|n|
  - Decrease use of parameter types #1453|n|
  - Ensure that the base_extrinsic weight does not change for Capacity transactions. #1455|n|
  - [Handles] Design doc cleanup #1435|n|
  - add CI base image #1468|n|
  - clear metadata mismatch label on Verify PR workflow start #1480|n|
  - switch Merge PR workflow to EKS runners #1459|n|
  - 1401 Build/run Frequency runtime without a relay chain #1464|n|
  - Bump robinraju/release-downloader from 1.7 to 1.8 #1473|n|
  - Update README.md #1489|n|
  - Update README.md #1490|n|
  - [chore] Add lint clippy::unwrap_used to modules and/or functions that must not panic #1476|n|
  - Feat/1463 vscode debug config #1492|n|
  - fix: added check for feature conflict #1493|n|
  - Update README.md #1491|n|
  - Update README.md #1495|n|
  - chore(capacity): address PR 827 comments #1372|n|
  - E2E tests on Testnet #1481|n|
  - switch Verify PR workflow to CI base image #1501|n|
  - Use standard srtool image version #1496|n|
  - build: Restore make-start to wasm execution; Add make-start-native; #1522|n|
  - New CLI parameter interface for block sealing #1520|n|
  - Build Optimization and Fix script warning #1523|n|
  - Replace production profile with release profile #1524|n|
  - update built binaries in CI #1518|n|
  - switch build binaries back to self-hosted runners #1527|n|
  - Update Readme #1525|n|
  - test(e2e-tests): Capacity staking from multiple msa to one provider; … #1497|n|
  - Message E2E Tests on Testnet #1514|n|
  - fix: Add patch to Cargo.toml to ensure correct multihash version #1531|n|
  - feat: add interval sealing mode for development node (without relay) #1533|n|
  - build: Update Rust: toolchain = nightly-2022-11-15 #1535|n|
  - add rust installation to ci base image #1536|n|
  - remove 3rd party rust toolchain action #1547|n|
  - Fix manual/interval sealing to work without empty blocks #1539|n|
  - Capacity E2E Tests on Testnet #1538|n|
'

clean="${changelog//[\`\[\]$'\n']/}"
echo "sanitized: $clean"

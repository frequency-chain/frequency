# Eth migrations tools
Some scripts and tools to facilitate the key migration

## Get Onchain msa id and keys
`npm run control-keys`


## Get Offchain Indexed msa id and keys
`npm run offchain-keys`

### Sort and Compare to see any discrepancies
1. `sort -t',' -k1,1n -k2,2 onchain-keys.txt >> onchain-keys-sorted.txt`
2. `sort -t',' -k1,1n -k2,2 offchain-keys.txt >> offchain-keys-sorted.txt`
3. `diff onchain-keys-sorted.txt offchain-keys-sorted.txt`

## Get Token keys
`npm run token-keys`

### Select token keys which are also control keys
`jq -c 'select(.msaId != 0)' tokens-keys.txt`

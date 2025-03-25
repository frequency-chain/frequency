# Eth migrations tools
Some scripts and tools to facilitate the key migration

## Get Onchain msa id and keys
`npm run control-keys --silent`


## Get Offchain Indexed msa id and keys
`npm run offchain-keys --silent`

### Sort and Compare to see any discrepancies
1. `sort -t',' -k1,1n -k2,2 onchain-keys.txt >> onchain-keys-sorted.txt`
2. `sort -t',' -k1,1n -k2,2 offchain-keys.txt >> offchain-keys-sorted.txt`
3. `diff onchain-keys-sorted.txt offchain-keys-sorted.txt`

## Get Token keys
`npm run token-keys --silent`

### Select token keys which are also control keys
`jq -c 'select(.msaId != 0)' tokens-keys.txt`

## Get the difference between on-chain control keys and Custodial Wallet
1.  fill in the CW DB connection string and other details
2. `npm run cw-compare`

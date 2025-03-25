# Eth migrations tools


# Get Onchain msa id and keys
`npm run control-keys`


# Get Offchain Indexed msa id and keys
`npm run offchain-keys`

## Sort and Compare to see any discrepancies
1. `sort -t',' -k1,1n -k2,2 onchain-keys.txt >> onchain-keys-sorted.txt`
2. `sort -t',' -k1,1n -k2,2 offchain-keys.txt >> offchain-keys-sorted.txt`
3. `diff onchain-keys-sorted.txt offchain-keys-sorted.txt`

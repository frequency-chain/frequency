# Frequency Parachain Node for Rococo/Mainnet

Frequency parachain node which connects to the public Rococo testnet or Mainnet networks.
Has no collating abilities.

## Run

```sh
# Connect to Mainnet
docker run --rm -p 9944:9944 -p 9933:9944 -p 30333:30333 frequencychain/parachain-node \
    --chain=frequency \
    --base-path=/chain-data \
    --rpc-external \
    --rpc-cors=all \
    --ws-external \
    --rpc-methods=safe

# Connect to Rococo Testnet
docker run --rm -p 9944:9944 -p 9933:9944 -p 30333:30333 frequencychain/parachain-node \
    --chain=frequency-rococo \
    --base-path=/chain-data \
    --rpc-external \
    --rpc-cors=all \
    --ws-external \
    --rpc-methods=safe
```

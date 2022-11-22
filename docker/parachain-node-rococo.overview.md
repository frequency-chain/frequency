# Frequency Parachain Node for Rococo Testnet

Frequency parachain node which connects to the public Rococo testnet network.
Has no collating abilities.

## Run

Start full chain node that connects to Rococo Testnet network:

```sh
docker run -p 9944:9944 -p 9933:9933 -p 30333:30333 frequencychain/parachain-node-rococo \
    --base-path=/chain-data \
    --rpc-external \
    --rpc-cors=all \
    --ws-external \
    --rpc-methods=safe
```

To view all available options and arguments:

```sh
docker run --rm frequencychain/parachain-node-rococo --help
```

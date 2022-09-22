# Frequency Parachain Node for Mainnet

Frequency parachain node which connects to the Mainnet network.
Has no collating abilities.

## Run

Start full chain node that connects to Mainnet network:

```sh
docker run --rm -p 9944:9944 -p 9933:9944 -p 30333:30333 frequencychain/parachain-node-mainnet \
    --base-path=/chain-data \
    --rpc-external \
    --rpc-cors=all \
    --ws-external \
    --rpc-methods=safe
```

To view all available options and arguments:

```sh
docker run --rm frequencychain/parachain-node-mainnet --help
```

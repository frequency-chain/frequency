# Frequency Parachain Node for Testnets

Frequency parachain node which connects to Frequency testnets:

- Frequency Paseo Testnet `--chain=frequency-paseo` (Default)
- Frequency Rococo Testnet `--chain=frequency-rococo`

To view all available options and arguments:

```sh
docker run --rm frequencychain/parachain-node-testnet:<version.tag> --help
```

## Run Full Node

### Frequency Paseo Testnet

Start full chain node that connects to Paseo Testnet network and syncs with warp:

```sh
docker run -p 9944:9944 -p 30333:30333 frequencychain/parachain-node-testnet:<version.tag> \
    --chain=frequency-paseo \
    --base-path=/chain-data \
    --rpc-external \
    --rpc-cors=all \
    --rpc-methods=safe \
    --sync=warp \
    -- \
    --sync=warp
```

### Frequency Rococo Testnet

Start full chain node that connects to Rococo Testnet network:

```sh
docker run -p 9944:9944 -p 30333:30333 frequencychain/parachain-node-testnet:<version.tag> \
    --chain=frequency-rococo \
    --base-path=/chain-data \
    --rpc-external \
    --rpc-cors=all \
    --rpc-methods=safe
```

## Storage

Remember that parachain nodes contain a full node of the relay chain as well, so plan available storage size accordingly.

Using [Volumes](https://docs.docker.com/storage/volumes/) or [Bind Mounts](https://docs.docker.com/storage/bind-mounts/) is suggested to maintain the `--base-path` between restarts.

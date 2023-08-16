# Frequency Parachain Node for Rococo Testnet

Frequency parachain node which connects to the public Rococo testnet network.

## Run Full Node

Start full chain node that connects to Rococo Testnet network:

```sh
docker run -p 9944:9944 -p 30333:30333 frequencychain/parachain-node-rococo:<version.tag> \
    --base-path=/chain-data \
    --rpc-external \
    --rpc-cors=all \
    --rpc-methods=safe
```

Remember that parachain nodes contain a full node of the relay chain as well, so plan available storage size accordingly.

Using [Volumes](https://docs.docker.com/storage/volumes/) or [Bind Mounts](https://docs.docker.com/storage/bind-mounts/) is suggested to maintain the `--base-path` between restarts.

To view all available options and arguments:

```sh
docker run --rm frequencychain/parachain-node-rococo:<version.tag> --help
```

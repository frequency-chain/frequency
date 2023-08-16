# Frequency Parachain Node for Mainnet

Frequency parachain node which connects to the Mainnet network.

## Run Full Node

Start full chain node that connects to Mainnet network:

```sh
docker run -p 9944:9944 -p 30333:30333 frequencychain/parachain-node-mainnet:<version.tag> \
    --base-path=/chain-data \
    --rpc-external \
    --rpc-cors=all \
    --rpc-methods=safe
```

Remember that parachain nodes contain a full node of the relay chain as well, so plan available storage size accordingly.

Using [Volumes](https://docs.docker.com/storage/volumes/) or [Bind Mounts](https://docs.docker.com/storage/bind-mounts/) is suggested to maintain the `--base-path` between restarts.

To view all available options and arguments:

```sh
docker run --rm frequencychain/parachain-node-mainnet:<version.tag> --help
```

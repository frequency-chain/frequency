# Frequency Collator Node in Local Only Sealing Mode

Runs just one collator node that will not connect to any other nodes.
Defaults to running in instant sealing mode where a block will be triggered when a transaction enters the validated transaction pool.
A "collator node" is a Frequency parachain node that is actively collating (aka forming blocks to submit to the relay chain, although in this case without a relay chain).

### Quick Run

```sh
docker run --rm -p 9944:9944 -p 30333:30333 frequencychain/instant-seal-node:<version.tag>
```


## Trigger Block Manually

If running in manual sealing mode or to form empty blocks in instant sealing mode, the `engine_createBlock` RPC can be used:

```sh
curl http://localhost:9944 -H "Content-Type:application/json;charset=utf-8" -d   '{ \
    "jsonrpc":"2.0", \
    "id":1, \
    "method":"engine_createBlock", \
    "params": [true, true] \
    }'
```


## Default Arguments

| Argument | Description |
| --- | --- |
| `--sealing=instant` | Manual sealing + automatically form a block each time a transaction enters the validated transaction pool |

### Run

Note: Docker `--rm` removes the volume when stopped.

```sh
docker run --rm -p 9944:9944 -p 30333:30333 frequencychain/instant-seal-node:<version.tag>
```

## Environment Variables

The following environment variables are supported by this image. The same behavior may be requested by overriding the command line arguments in the `CMD` of the container; however, certain use cases (GitHub Actions) do not support overriding `CMD` when instantiating a container-based service in a workflow. In such a case, injecting these environment variables is a viable workaround.

| Environmnet Variable | Possible Values | Description |
| --- | --- | --- |
| `SEALING_MODE` | `instant`, `interval`, `manual` | Overrides `--sealing=SEALING_MODE` |
| `SEALING_INTERVAL` | integer > 0 | Adds `--sealing-interval=SEALING_INTERVAL`. Number of seconds between block in `interval` sealing mode |
| `CREATE_EMPTY_BLOCKS` | `true` | Add `--sealing-create-empty-blocks`. Whether to form empty blocks on the interval in `interval` sealing mode |


## Overriding Arguments

| Argument | Description |
| --- | --- |
| `--sealing=manual` | Only form a block when `engine_createBlock` RPC is called |
| `--help` | See all the options possible |

### Run

```sh
docker run --rm -p 9944:9944 -p 30333:30333 frequencychain/instant-seal-node:<version.tag> -- --manual-seal
```

| **Node**                |             **Ports**             | **Explorer URL**                                                                          |
| ----------------------- | :-------------------------------: | ----------------------------------------------------------------------------------------- |
| Frequency Local-Only Node | ws and rpc :`9944`, p2p:`3033`  | [127.0.0.1:9944](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer) |

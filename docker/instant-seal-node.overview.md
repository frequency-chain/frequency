# Frequency Collator Node in Local Only Sealing Mode

Runs just one collator node that will not connect to any other nodes.
Defaults to running in instant sealing mode where a block will be triggered when a transaction enters the validated transaction pool.
A "collator node" is a Frequency parachain node that is actively collating (aka forming blocks to submit to the relay chain, although in this case without a relay chain).

### Quick Run

```sh
docker run --rm -p 9944:9944 -p 9933:9933 -p 30333:30333 frequencychain/instant-seal-node:<version.tag>
```


## Trigger Block Manually

If running in manual sealing mode or to form empty blocks in instant sealing mode, the `engine_createBlock` RPC can be used:

```sh
curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d   '{ \
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
docker run --rm -p 9944:9944 -p 9933:9933 -p 30333:30333 frequencychain/instant-seal-node:<version.tag>
```

## Overriding Arguments

| Argument | Description |
| --- | --- |
| `--sealing=manual` | Only form a block when `engine_createBlock` RPC is called |
| `--help` | See all the options possible |

### Run

```sh
docker run --rm -p 9944:9944 -p 9933:9933 -p 30333:30333 frequencychain/instant-seal-node:<version.tag> -- --manual-seal
```

| **Node**                |             **Ports**             | **Explorer URL**                                                                          |
| ----------------------- | :-------------------------------: | ----------------------------------------------------------------------------------------- |
| Frequency Local-Only Node | ws:`9944`, rpc`:9933`, p2p:`3033` | [127.0.0.1:9944](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer) |

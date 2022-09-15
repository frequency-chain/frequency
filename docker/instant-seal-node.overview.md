# Frequency Collator Node in Instant Seal Mode

Runs just one collator node in instant seal mode.
A "collator node" is a Frequency parachain node that is actively collating (aka forming blocks to submit to the relay chain).
The instant seal mode allows a blockchain node to author a block
as soon as it goes into a queue.
This is also a great option to run with an example client.

![](https://github.com/LibertyDSNP/frequency/blob/main/docs/images/local-dev-env-option-1.jpg?raw=true)

## Run

```sh
docker run --rm -p 9944:9944 -p 9933:9944 -p 30333:30333 frequencychain/instant-seal-node
```

| **Node**                |             **Ports**             | **Explorer URL**                                                                          |
| ----------------------- | :-------------------------------: | ----------------------------------------------------------------------------------------- |
| Frequency Collator Node | ws:`9944`, rpc`:9933`, p2p:`3033` | [127.0.0.1:9944](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer) |

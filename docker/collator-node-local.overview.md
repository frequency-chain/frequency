# Frequency Collator Node for Local Relay Chain

Runs one collator node that connects to the local relay chain consisting of two validator nodes.

![](https://github.com/LibertyDSNP/frequency/blob/main/docs/images/local-dev-env-option-2.jpg?raw=true)

## Run

1.  Checkout project and generate local spec

    ```
    git clone git@github.com:LibertyDSNP/frequency.git
    ```

1.  Generate chain specs

    ```
    cd frequency
    make specs-rococo-2000
    ```

1.  Start relay chain and collator node

    ```sh

    make start-frequency-docker
    ```

1.  Stop all nodes

    ```sh
    make stop-frequency-docker
    ```

| **Node**             | **Ports**                           | **Explorer URL**                                                                          |
| -------------------- | ----------------------------------- | ----------------------------------------------------------------------------------------- |
| Frequency Relay Node | ws:`9944`, rpc`:9933`, p2p:`30333`  | [127.0.0.1:9944](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/explorer) |
| Alice Relay Node     | ws:`:9946`, rpc`:9935`, p2p:`30335` | [127.0.0.1:9946](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9946#/explorer) |
| Bob Relay Node       | ws:`:9947`, rpc`:9936`, p2p:`30336` | [127.0.0.1:9947](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9947#/explorer) |

```

```

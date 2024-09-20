# State Copy Tool

Tools to help copy state from one Frequency Chain to another.

## Testnet Schemas Match

To maintain alignment with Mainnet, when a new schema is deployed on Mainnet, Testnet data and Dev Genesis are set to have those exact same schemas.

1. In the Frequency codebase: `cd tools/state-copy`
2. `npm i`
3. Edit `copy.mjs`
  - Use the Testnet `DEST_URL`: `const DEST_URL = "wss://0.rpc.testnet.amplica.io";`
  - Update the`const SUDO_URI = "//Alice";` to be the SUDO key for Testnet
4. `npm run schemas`
5. Remove the changes

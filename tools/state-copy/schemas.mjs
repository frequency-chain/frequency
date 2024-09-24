// This small nodejs script will pull the schema pallet storage from mainnet and then load it onto another chain.
//
// WARNING: This will move ALL storage values, so if there are some you do NOT want, use the FILTER_OUT or another setup.

import { copy } from "./copy.mjs";

// Set up sudo account (assumes //Alice has sudo access)
const SUDO_URI = "//Alice";

// Testnet
// const DEST_URL = "wss://0.rpc.testnet.amplica.io";
// const SOURCE_URL = "wss://0.rpc.testnet.amplica.io";

// Localhost
const DEST_URL = "ws://localhost:9944";

// Mainnet
const SOURCE_URL = "wss://1.rpc.frequency.xyz";
const STORAGE_KEY = "0xeec6f3c13d26ae2507c99b6751e19e76";
const FILTER_OUT = [
  "0xeec6f3c13d26ae2507c99b6751e19e76d5d9c370c6c8aee1116ee09d6811b0d5", // governanceSchemaModelMaxBytes
  "0xeec6f3c13d26ae2507c99b6751e19e764e7b9012096b41c4eb3aaf947f6ea429", // palletVersion
  // Comment this out to INCLUDE setting the currentSchemaIdentifierMaximum from the SOURCE
  "0xeec6f3c13d26ae2507c99b6751e19e765b81a4f27a1e406724e3a53d909f29cd", // currentSchemaIdentifierMaximum
];

async function main() {
  try {
    await copy(SOURCE_URL, DEST_URL, STORAGE_KEY, SUDO_URI, FILTER_OUT);

    process.exit(0);
  } catch (error) {
    console.error("Error:", error);
    process.exit(1);
  }
}

main();

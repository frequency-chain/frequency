// This small nodejs script will pull the schemas from mainnet and then load them onto local

import { copy } from "./copy.mjs";

const SOURCE_URL = "wss://1.rpc.frequency.xyz";
const DEST_URL = "ws://127.0.0.1:9944";
const STORAGE_KEY = "0xeec6f3c13d26ae2507c99b6751e19e76";
const FILTER_OUT = [
  "0xeec6f3c13d26ae2507c99b6751e19e76d5d9c370c6c8aee1116ee09d6811b0d5", // governanceSchemaModelMaxBytes
  "0xeec6f3c13d26ae2507c99b6751e19e764e7b9012096b41c4eb3aaf947f6ea429", // palletVersion
];

async function main() {
  try {
    await copy(SOURCE_URL, DEST_URL, STORAGE_KEY, FILTER_OUT);
    process.exit(0);
  } catch (error) {
    console.error("Error:", error);
    process.exit(1);
  }
}

main();

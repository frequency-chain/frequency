// This small nodejs script will pull the schemas from mainnet and then output them in Genesis Schema format

import { getSchemas } from "./getSchema.mjs";

const SOURCE_URL = "wss://1.rpc.frequency.xyz";

async function main() {
  try {
    await getSchemas(SOURCE_URL);
    process.exit(0);
  } catch (error) {
    console.error("Error:", error);
    process.exit(1);
  }
}

main();

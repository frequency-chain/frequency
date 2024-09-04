import { ApiPromise, WsProvider, Keyring } from "@polkadot/api";

import { readFileSync } from "fs";

async function main() {
  try {
    const endpoint = process.argv[2];
    const seed = process.argv[3];
    const wasmFile = process.argv[4];
    const provider = new WsProvider(endpoint);
    const api = await ApiPromise.create({ provider });
    const keyring = new Keyring({ type: "sr25519" });
    const sudo = keyring.addFromUri(seed);
    console.log(`--- Submitting extrinsic to upgrade dev node ---`);
    let wasm;
    try {
      wasm = "0x" + readFileSync(wasmFile).toString("hex");
    } catch (err) {
      console.error(err);
      throw err;
    }
    const sudoCall = await api.tx.sudo
      .sudoUncheckedWeight(api.tx.system.setCodeWithoutChecks(wasm), { refTime: 1 })
      .signAndSend(sudo, (result) => {
        console.log(`Current status is ${result.status}`);
        if (result.status.isInBlock) {
          console.log(`Transaction included at blockHash ${result.status.asInBlock}`);
          console.log("Waiting for finalization...");
        } else if (result.status.isFinalized) {
          console.log(`Transaction finalized at blockHash ${result.status.asFinalized}`);
          sudoCall();
          process.exit();
        } else if (result.isError) {
          console.log(`Transaction Error`);
          process.exit();
        }
      });
  } catch (error) {
    console.log("error:", error);
  }
}

main();

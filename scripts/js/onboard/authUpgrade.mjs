import { ApiPromise, WsProvider, Keyring } from "@polkadot/api";

async function main() {
  try {
    const endpoint = process.argv[2];
    const seed = process.argv[3];
    const hash = process.argv[4];
    const provider = new WsProvider(endpoint);
    const api = await ApiPromise.create({ provider });
    const keyring = new Keyring({ type: "sr25519" });
    const sudo = keyring.addFromUri(seed);
    console.log(`--- Submitting extrinsic to authorize testnet-2000 upgrade ---`);
    const sudoCall = await api.tx.sudo
      .sudo(api.tx.parachainSystem.authorizeUpgrade(hash, false))
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

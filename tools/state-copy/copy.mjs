import { ApiPromise, WsProvider } from "@polkadot/api";
import { Keyring } from "@polkadot/keyring";

const keyring = new Keyring({ type: "sr25519" });

async function getKeys(api, storageKey) {
  const pageSize = 500;
  const result = [];
  let startKey = "";
  while (true) {
    const page = await api.rpc.state.getKeysPaged(storageKey, pageSize, startKey);
    result.push(...page.map((x) => x.toString()));
    if (page.length === 0) {
      break;
    }

    startKey = page[page.length - 1].toString();
  }
  return result;
}

export async function copy(sourceUrl, destUrl, storageKey, filterKeys = []) {
  // Connect to the state source
  const sourceProvider = new WsProvider(sourceUrl);
  const sourceApi = await ApiPromise.create({ provider: sourceProvider });

  // Connect to destination source
  const destProvider = new WsProvider(destUrl);
  const destApi = await ApiPromise.create({ provider: destProvider });

  console.log("Connected to both networks");

  // Get all keys from the specified pallet
  const keys = (await getKeys(sourceApi, storageKey)).filter((k) => !filterKeys.includes(k));
  console.log(`Found ${keys.length} keys under ${storageKey}...`);

  // Fetch values for all keys
  const storageKV = await Promise.all(
    keys.map(async (key) => {
      const value = await sourceApi.rpc.state.getStorage(key);
      return [key, value.toHex()];
    }),
  );

  console.log("Fetched all values", storageKV);

  // Set up sudo account (assumes //Alice has sudo access)
  const sudoAccount = keyring.createFromUri("//Alice");

  // Prepare and send sudo.sudo(system.setStorage()) call
  const sudoCall = destApi.tx.sudo.sudo(destApi.tx.system.setStorage(storageKV));

  console.log("Submitting sudo call to set storage...");
  await new Promise(async (resolve, reject) => {
    const unsub = await sudoCall.signAndSend(sudoAccount, ({ status, events }) => {
      if (status.isInBlock || status.isFinalized) {
        console.log(
          `Block hash: ${(status.isInBlock && status.asInBlock) || (status.isFinalized && status.asFinalized)}`,
        );
        const success = events.find((x) => destApi.events.system.ExtrinsicSuccess.is(x.event));
        const failure = events.find((x) => destApi.events.system.ExtrinsicFailed.is(x.event));
        unsub();
        if (success && !failure) {
          console.log("State copy successful");
          resolve();
        } else {
          console.error("State copy FAILED!");
          reject();
        }
      }
    });
  });
}

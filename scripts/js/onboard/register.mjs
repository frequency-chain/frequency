import { ApiPromise, WsProvider, Keyring } from "@polkadot/api";

const run = async () => {
  try {
    console.log("Parsing Args ...");

    const endpoint = process.argv[2];
    const seed = process.argv[3];

    const wsProvider = new WsProvider(endpoint);

    const api = await ApiPromise.create({
      provider: wsProvider,
    });

    const keyring = new Keyring({ type: "sr25519" });

    const nextParaId = (await api.query.registrar.nextFreeParaId()).toString();

    console.log(" ℹ - Next Available Parachain ID: ", nextParaId);

    const alice = keyring.addFromUri(seed);

    await reservePara(alice, api, nextParaId);
  } catch (error) {
    console.log("error:", error);
  }
};

const reservePara = async (account, api, paraID) => {
  const nonce = Number((await api.query.system.account(account.address)).nonce);
  // Reserve Parachain ID
  console.log(`🤖 - Reserving Parachain ID`);
  let signAndSend = await api.tx.registrar.reserve().signAndSend(account, { nonce: nonce }, async ({ status }) => {
    if (status.isFinalized) {
      console.log(`✔️  - Finalized at block hash #${status.asFinalized.toString()} \n`);
      //await registerPara(account, api, paraID);
      signAndSend();
      process.exit();
    }
  });
};

const registerPara = async (account, api, paraID) => {
  const nonce = Number((await api.query.system.account(account.address)).nonce);
  // Register Parachain
  console.log(`🤖 - Registering Parachain`);
  let signAndSend = await api.tx.registrar
    .register(paraID, "0x11", "0x11")
    .signAndSend(account, { nonce: nonce }, async ({ status }) => {
      if (status.isFinalized) {
        console.log(`✔️  - Finalized at block hash #${status.asFinalized.toString()} \n`);
        signAndSend();
        process.exit();
      }
    });
};

run();

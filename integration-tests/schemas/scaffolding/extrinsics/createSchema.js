const { ApiPromise, WsProvider } = require("@polkadot/api");
const { options } = require("@frequency-chain/api-augment");
const { Keyring } = require("@polkadot/api");

async function getFrequencyAPI() {
    let DEPLOY_SCHEMA_ENDPOINT_URL = process.env.DEPLOY_SCHEMA_ENDPOINT_URL;
    if (DEPLOY_SCHEMA_ENDPOINT_URL === undefined) {
      // One would think that localhost would also work here but it doesn't consistently.
      DEPLOY_SCHEMA_ENDPOINT_URL = "ws://127.0.0.1:9944";
    }
    const DefaultWsProvider = new WsProvider(DEPLOY_SCHEMA_ENDPOINT_URL);

    // The "options" parameter pulls in the Frequency API extrinsics
    const api = await ApiPromise.create({
      provider: DefaultWsProvider,
      ...options,
    });
    await api.isReady;
    return api;
}

const getSignerAccountKeys = () => {
    const keyring = new Keyring();

    let DEPLOY_SCHEMA_ACCOUNT_URI = process.env.DEPLOY_SCHEMA_ACCOUNT_URI;
    if (DEPLOY_SCHEMA_ACCOUNT_URI === undefined) {
        DEPLOY_SCHEMA_ACCOUNT_URI = "//Alice";
    }
    return keyring.addFromUri(DEPLOY_SCHEMA_ACCOUNT_URI, {}, "sr25519");
};

const createSchema = async (payload, modelType, payloadLocation) => {
    const api = await getFrequencyAPI();
    const signerAccountKeys = getSignerAccountKeys();
  
    // Retrieve the current account nonce so we can increment it when submitting transactions
    let nonce = (await api.rpc.system.accountNextIndex(signerAccountKeys.address)).toNumber();
  
    console.log("Attempting to register schema.");

    const tx = await api.tx.schemas.createSchema(JSON.stringify(payload), modelType, payloadLocation);

    return new Promise((resolve, reject) => {
        tx.signAndSend(signerAccountKeys, { nonce: nonce++ }, ({status, events}) => {
            if (status.isFinalized) {
                console.log(events.forEach(({event}) => console.log(event.method)));
                const schemaRegisteredEvent = events.find(({ event }) => event.section === "schemas" && event.method === "SchemaRegistered");
                const successEvent = events.find(({ event }) => event.section === "system" && event.method === "ExtrinsicSuccess");
    
                let success = typeof(schemaRegisteredEvent) !== "undefined" && typeof(successEvent) !== "undefined";
                
                let schemaId;
                if (success) { schemaId = schemaRegisteredEvent.event.data[1] }
                resolve({success, schemaId});
            }
        });
    });
};

module.exports = createSchema;
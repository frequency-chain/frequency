const { ApiPromise, WsProvider } = require("@polkadot/api");
const { options } = require("@frequency-chain/api-augment");
const { Keyring } = require("@polkadot/api");

exports.getFrequencyAPI = async () => {
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

exports.getSignerAccountKeys = () => {
    const keyring = new Keyring();

    let DEPLOY_SCHEMA_ACCOUNT_URI = process.env.DEPLOY_SCHEMA_ACCOUNT_URI;
    if (DEPLOY_SCHEMA_ACCOUNT_URI === undefined) {
        DEPLOY_SCHEMA_ACCOUNT_URI = "//Alice";
    }
    return keyring.addFromUri(DEPLOY_SCHEMA_ACCOUNT_URI, {}, "sr25519");
};
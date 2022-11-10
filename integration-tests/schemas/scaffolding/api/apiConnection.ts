import { ApiPromise, ApiRx, Keyring, WsProvider } from "@polkadot/api";
import { options } from "@frequency-chain/api-augment";

export const getFrequencyAPI = async (): Promise<ApiPromise> => {
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

export const getSignerAccountKeys = () => {
    const keyring: Keyring = new Keyring();

    let DEPLOY_SCHEMA_ACCOUNT_URI = process.env.DEPLOY_SCHEMA_ACCOUNT_URI;
    if (DEPLOY_SCHEMA_ACCOUNT_URI === undefined) {
        DEPLOY_SCHEMA_ACCOUNT_URI = "//Alice";
    }
    return keyring.addFromUri(DEPLOY_SCHEMA_ACCOUNT_URI, {}, "sr25519");
};
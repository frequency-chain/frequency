import { options } from "@frequency-chain/api-augment";
import { ApiRx, WsProvider, Keyring } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import { firstValueFrom } from "rxjs";

export async function connect(providerUrl): Promise<{api: ApiRx, keys: KeyringPair}> {
    let url = providerUrl;
    if (!providerUrl) {
        url = "ws://127.0.0.1:9944"
    }
    const provider = new WsProvider(url);
    const apiObservable = ApiRx.create({ provider, ...options });
    const keyring = new Keyring({ type: "sr25519" });
    const keys = keyring.addFromUri("//Alice");

    const api = await firstValueFrom(apiObservable);

    return { api, keys }
}

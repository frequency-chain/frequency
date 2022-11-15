import { options } from "@frequency-chain/api-augment";
import { ApiRx, WsProvider, Keyring } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import { firstValueFrom } from "rxjs";

export async function connect(providerUrl): Promise<{api: ApiRx, keys: KeyringPair}> {
    const provider = new WsProvider(providerUrl);
    const apiObservable = ApiRx.create({ provider, ...options });
    const keyring = new Keyring({ type: "sr25519" });
    const keys = keyring.addFromUri("//Alice");

    const api = await firstValueFrom(apiObservable);

    return { api, keys }
}

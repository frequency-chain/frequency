import { options } from "@frequency-chain/api-augment";
import { ApiRx, WsProvider, ApiPromise } from "@polkadot/api";
import { firstValueFrom } from "rxjs";
import { env } from "./env";

export async function connect(providerUrl?: string | string[] | undefined): Promise<ApiRx> {
    const provider = new WsProvider(providerUrl || env.providerUrl);
    const apiObservable = ApiRx.create({ provider, ...options });
    return firstValueFrom(apiObservable);
}

export async function connectPromise(providerUrl?: string | string[] | undefined): Promise<ApiPromise> {
    const provider = new WsProvider(providerUrl || env.providerUrl);
    const api = await ApiPromise.create({ provider, ...options });
    await api.isReady;
    return api;
}

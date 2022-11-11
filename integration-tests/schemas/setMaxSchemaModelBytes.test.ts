import { ApiRx, WsProvider, Keyring } from "@polkadot/api";
import assert from "assert";
import { filter, firstValueFrom, mergeMap, Observable } from "rxjs";
import { groupEventsByKey } from "./scaffolding/helpers";

describe("#setMaxSchemaModelBytes", () => {
    let apiObservable: Observable<ApiRx>;
    let keys: any;

    beforeEach(() => {
        const provider = new WsProvider("ws://127.0.0.1:9944");
        apiObservable = ApiRx.create({ provider });
        const keyring = new Keyring({ type: "sr25519" });
        keys = keyring.addFromUri("//Alice");
    })

    it("should fail to set the schema size because of lack of root authority", async () => {
        const chainEvents = await firstValueFrom(
            apiObservable.pipe(
                mergeMap((api) => api.tx.schemas.setMaxSchemaModelBytes(1000000).signAndSend(keys).pipe(
                filter(({status}) => status.isInBlock),
                groupEventsByKey()
            ))))

        assert.notEqual(chainEvents["system.ExtrinsicFailed"], undefined);
        assert.equal(chainEvents["system.ExtrinsicSuccess"], undefined);

    }).timeout(1000000);

    // NOTE: We need a root account to test the positive case
});

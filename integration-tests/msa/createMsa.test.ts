import "@frequency-chain/api-augment";
import assert from "assert";
import { ApiRx } from "@polkadot/api";
import { connect } from "../scaffolding/apiConnection"
import { filter, firstValueFrom } from "rxjs";
import { groupEventsByKey } from "../scaffolding/helpers";
import { KeyringPair } from "@polkadot/keyring/types";

describe("Create Accounts", () => {
    let api: ApiRx;
    let keys: KeyringPair;

    before(async () => {
        let {api: connectApi, keys: connectKeys} = await connect(process.env.WS_PROVIDER_URL);
        api = connectApi
        keys = connectKeys
    })

    after(() => {
        api.disconnect()
    })

    it("should successfully create an MSA account", async () => {
        const chainEvents = await firstValueFrom(api.tx.msa.create().signAndSend(keys).pipe(
                filter(({status}) => status.isInBlock || status.isFinalized),
                groupEventsByKey()))

        assert.equal(chainEvents["system.ExtrinsicFailed"], undefined);
        assert.notEqual(chainEvents["system.ExtrinsicSuccess"], undefined);
        assert.notEqual(chainEvents["msa.MsaCreated"], undefined);
        assert.notEqual(chainEvents["transactionPayment.TransactionFeePaid"], undefined);
    }).timeout(15000);
})

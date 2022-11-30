import "@frequency-chain/api-augment";
import assert from "assert";
import { ApiRx } from "@polkadot/api";
import { connect, createKeys } from "../scaffolding/apiConnection"
import { groupEventsByKey, signPayloadSr25519 } from "../scaffolding/helpers";
import { KeyringPair } from "@polkadot/keyring/types";
import { addPublicKeyToMsa, createMsa, createSchema, deletePublicKey } from "../scaffolding/extrinsicHelpers";
import { AVRO_GRAPH_CHANGE } from "../schemas/fixtures/avroGraphChangeSchemaType";
import { filter, firstValueFrom } from "rxjs";

describe("Create Accounts", () => {
    let api: ApiRx;
    let keys: KeyringPair;

    before(async () => {
        let connectApi = await connect(process.env.WS_PROVIDER_URL);
        api = connectApi
    })

    after(() => {
        api.disconnect()
    })

    // NOTE: We will need a sustainable way to create new keys for every test,
    // since there is only one node instance per test suite.
    it("should successfully create an MSA account", async () => {
        keys = createKeys("//Alice")
        const chainEvents = await createMsa(api, keys)

        assert.equal(chainEvents["system.ExtrinsicFailed"], undefined);
        assert.notEqual(chainEvents["system.ExtrinsicSuccess"], undefined);
        assert.notEqual(chainEvents["msa.MsaCreated"], undefined);
        assert.notEqual(chainEvents["transactionPayment.TransactionFeePaid"], undefined);
    }).timeout(15000)

    it("should successfully mimic a user's path using tokens", async () => {
        keys = createKeys("//Charlie")
        const createMsaEvents = await createMsa(api, keys);
        assert.notEqual(createMsaEvents["msa.MsaCreated"], undefined);

        const msaId = createMsaEvents["msa.MsaCreated"][0]
        const newKeys: KeyringPair = createKeys("//Bob");
        const payload = {msaId: msaId, newPublicKey: newKeys.publicKey, expiration: 12}

        const addKeyData = api.registry.createType("PalletMsaAddKeyData", payload); 

        const ownerSig = signPayloadSr25519(keys, addKeyData);
        const newSig = signPayloadSr25519(newKeys, addKeyData);

        const publicKeyEvents = await addPublicKeyToMsa(api, keys, ownerSig, newSig, payload);

        assert.notEqual(publicKeyEvents["msa.PublicKeyAdded"], undefined);

        const deleteEvents = await deletePublicKey(api, keys, newKeys.publicKey);

        assert.notEqual(deleteEvents["msa.PublicKeyDeleted"], undefined);

        const createSchemaEvents = await createSchema(api, keys, AVRO_GRAPH_CHANGE, "AvroBinary", "OnChain");
        assert.notEqual(createSchemaEvents["schemas.SchemaCreated"], undefined);
        
        const retireMsaEvents = await firstValueFrom(
            api.tx.msa.retireMsa().signAndSend(keys).pipe(
                filter(({status}) => status.isInBlock || status.isFinalized),
                groupEventsByKey()))
        
        assert.notEqual(retireMsaEvents["msa.PublicKeyDeleted"], undefined);
        assert.notEqual(retireMsaEvents["msa.MsaRetired"], undefined);
    }).timeout(15000)
})

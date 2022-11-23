import "@frequency-chain/api-augment";
import assert from "assert";
import { ApiRx } from "@polkadot/api";
import { connect, createKeys } from "../scaffolding/apiConnection"
import { signPayloadSr25519 } from "../scaffolding/helpers";
import { KeyringPair } from "@polkadot/keyring/types";
import { addPublicKeyToMsa, createMsa, createSchema, deletePublicKey } from "../scaffolding/extrinsicHelpers";
import { AVRO_GRAPH_CHANGE } from "../schemas/fixtures/avroGraphChangeSchemaType";

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

    it("should successfully create an MSA account", async () => {
        keys = createKeys("//Charlie")
        const chainEvents = await createMsa(api, keys)

        assert.equal(chainEvents["system.ExtrinsicFailed"], undefined);
        assert.notEqual(chainEvents["system.ExtrinsicSuccess"], undefined);
        assert.notEqual(chainEvents["msa.MsaCreated"], undefined);
        assert.notEqual(chainEvents["transactionPayment.TransactionFeePaid"], undefined);
    }).timeout(15000)

    it("should successfully mimic a user's path using tokens", async () => {
        keys = createKeys("//Alice")
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
    }).timeout(15000)
})

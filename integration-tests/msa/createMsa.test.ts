import "@frequency-chain/api-augment";
import assert from "assert";
import { ApiRx } from "@polkadot/api";
import { connect } from "../scaffolding/apiConnection"
import { AccountFundingInputs, createAccount, createAndFundAccount, generateFundingInputs, txAccountingHook, signPayloadSr25519 } from "../scaffolding/helpers";
import { KeyringPair } from "@polkadot/keyring/types";
import { addPublicKeyToMsa, createMsa, createSchema, deletePublicKey, signAndSend } from "../scaffolding/extrinsicHelpers";
import { AVRO_GRAPH_CHANGE } from "../schemas/fixtures/avroGraphChangeSchemaType";

describe("Create Accounts", function () {
    this.timeout(15000);

    let fundingInputs: AccountFundingInputs;

    let api: ApiRx;

    before(async function () {
        let connectApi = await connect(process.env.WS_PROVIDER_URL);
        api = connectApi
        fundingInputs = generateFundingInputs(api, this.title);
    })

    after(async function () {
        await txAccountingHook(api, fundingInputs.context);
        await api.disconnect()
    })

    // NOTE: We will need a sustainable way to create new keys for every test,
    // since there is only one node instance per test suite.
    it("should successfully create an MSA account", async function () {
        const keys = (await createAndFundAccount(fundingInputs)).newAccount;
        const chainEvents = await createMsa(api, keys);

        assert.notEqual(chainEvents["system.ExtrinsicSuccess"], undefined);
        assert.notEqual(chainEvents["msa.MsaCreated"], undefined);
        assert.notEqual(chainEvents["transactionPayment.TransactionFeePaid"], undefined);
    });

    it("should successfully mimic a user's path using tokens", async function () {
        const keys = (await createAndFundAccount(fundingInputs)).newAccount;
        const createMsaEvents = await createMsa(api, keys);
        const msaId = createMsaEvents["msa.MsaCreated"][0]
        assert.notEqual(msaId, undefined);

        const newKeys: KeyringPair = createAccount();
        const payload = { msaId: msaId, newPublicKey: newKeys.publicKey, expiration: 50 }

        const addKeyData = api.registry.createType("PalletMsaAddKeyData", payload);

        const ownerSig = signPayloadSr25519(keys, addKeyData);
        const newSig = signPayloadSr25519(newKeys, addKeyData);

        const publicKeyEvents = await addPublicKeyToMsa(api, keys, ownerSig, newSig, payload);

        assert.notEqual(publicKeyEvents["msa.PublicKeyAdded"], undefined, 'should have added public key');

        const deleteEvents = await deletePublicKey(api, keys, newKeys.publicKey);

        assert.notEqual(deleteEvents["msa.PublicKeyDeleted"], undefined, 'should have deleted public key');

        const createSchemaEvents = await createSchema(api, keys, AVRO_GRAPH_CHANGE, "AvroBinary", "OnChain");
        assert.notEqual(createSchemaEvents["schemas.SchemaCreated"], undefined, 'should have created schema');

        const retireMsaEvents = await signAndSend(() => api.tx.msa.retireMsa(), keys);

        assert.notEqual(retireMsaEvents["msa.PublicKeyDeleted"], undefined, 'should have deleted public key (retired)');
        assert.notEqual(retireMsaEvents["msa.MsaRetired"], undefined, 'should have retired msa');
    });
})

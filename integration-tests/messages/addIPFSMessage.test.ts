import "@frequency-chain/api-augment";
import { ApiRx } from "@polkadot/api";
import type { Codec } from '@polkadot/types-codec/types';
import { connect, createKeys } from "../scaffolding/apiConnection"
import { KeyringPair } from "@polkadot/keyring/types";
import { createSchema, addIPFSMessage, createMsa } from "../scaffolding/extrinsicHelpers";
import { PARQUET_BROADCAST } from "../schemas/fixtures/parquetBroadcastSchemaType";
import assert from "assert";
import { createAndFundAccount, DevAccounts, EventMap, INITIAL_FUNDING, showTotalCost } from "../scaffolding/helpers";

describe("Add Offchain Message", function () {
    this.timeout(15000);

    const context = this.title;
    const amount = INITIAL_FUNDING;
    const source = DevAccounts.Alice;

    let api: ApiRx;
    let keys: KeyringPair;
    let schemaId: Codec;

    // This is how the IPFS data was created:
    //
    // echo "This is a test of Frequency." > frequency_test
    // % ipfs add frequency_test
    // added QmYzm8KGxRHr7nGn5g5Z9Zv9r8nN5WNn7Ajya6x7RxmAB1 frequency_test
    // % wc -c frequency_test
    // 29 frequency_test
    const ipfs_cid = "QmYzm8KGxRHr7nGn5g5Z9Zv9r8nN5WNn7Ajya6x7RxmAB1";
    const ipfs_payload_data = "This is a test of Frequency.";
    const ipfs_payload_len = ipfs_payload_data.length;

    before(async function () {
        let connectApi = await connect(process.env.WS_PROVIDER_URL);
        api = connectApi

        const accounts = await createAndFundAccount({ api, amount, source, context });
        keys = accounts.newAccount;

        // Create a new MSA
        await createMsa(api, keys);

        // Create a schema
        const createSchemaEvents = await createSchema(api, keys, PARQUET_BROADCAST, "Parquet", "IPFS");
        const event = createSchemaEvents["schemas.SchemaCreated"];
        schemaId = event[1];
    })

    after(async function () {
        await showTotalCost(api, context);
        await api.disconnect()
    })

    it('should fail if MSA is not valid', async function () {
        const bogusKey = createKeys(DevAccounts.Eve);
        await assert.rejects(addIPFSMessage(api, bogusKey, schemaId, ipfs_cid, ipfs_payload_len + 1), {
            name: 'InvalidMessageSourceAccount',
            section: 'messages',
        });
    });

    it('should fail if schema does not exist', async function () {
        await assert.rejects(addIPFSMessage(api, keys, 2, ipfs_cid, ipfs_payload_len + 1), {
            name: 'InvalidSchemaId',
            section: 'messages',
        });
    });

    it("should successfully add an IPFS message", async function () {
        const chainEvents: EventMap = await addIPFSMessage(api, keys, schemaId, ipfs_cid, ipfs_payload_len + 1);

        assert.notEqual(chainEvents["system.ExtrinsicSuccess"], undefined);
    });
})


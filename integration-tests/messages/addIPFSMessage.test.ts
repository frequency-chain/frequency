import "@frequency-chain/api-augment";
import { KeyringPair } from "@polkadot/keyring/types";
import { PARQUET_BROADCAST } from "../schemas/fixtures/parquetBroadcastSchemaType";
import assert from "assert";
import { createAndFundKeypair, devAccounts } from "../scaffolding/helpers";
import { ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { u16 } from "@polkadot/types";

describe("Add Offchain Message", function () {
    let keys: KeyringPair;
    let schemaId: u16;

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
        keys = await createAndFundKeypair();

        // Create a new MSA
        const createMsa = ExtrinsicHelper.createMsa(keys);
        await createMsa.fundAndSend();

        // Create a schema
        const createSchema = ExtrinsicHelper.createSchema(keys, PARQUET_BROADCAST, "Parquet", "IPFS");
        const [event] = await createSchema.fundAndSend();
        if (event && createSchema.api.events.schemas.SchemaCreated.is(event)) {
            [, schemaId] = event.data;
        }
    })

    it('should fail if insufficient funds', async function () {
        await assert.rejects(ExtrinsicHelper.addIPFSMessage(keys, schemaId, ipfs_cid, ipfs_payload_len + 1).signAndSend(), {
            message: /Inability to pay some fees/,
        });
    })

    it('should fail if MSA is not valid', async function () {
        const accountWithNoMsa = devAccounts[0].keys;
        await assert.rejects(ExtrinsicHelper.addIPFSMessage(accountWithNoMsa, schemaId, ipfs_cid, ipfs_payload_len + 1).signAndSend(), {
            name: 'InvalidMessageSourceAccount',
            section: 'messages',
        });
    });

    it('should fail if schema does not exist', async function () {
        // If we ever create more than 999 schemas in a test suite/single Frequency instance, this test will fail.
        const f = ExtrinsicHelper.addIPFSMessage(keys, 999, ipfs_cid, ipfs_payload_len + 1);
        await assert.rejects(f.fundAndSend(), {
            name: 'InvalidSchemaId',
            section: 'messages',
        });
    });

    it("should successfully add an IPFS message", async function () {
        const f = ExtrinsicHelper.addIPFSMessage(keys, schemaId, ipfs_cid, ipfs_payload_len + 1);
        const [event] = await f.fundAndSend();

        assert.notEqual(event, undefined, "should have returned a MessagesStored event");
        if (event && f.api.events.messages.MessagesStored.is(event)) {
            assert.deepEqual(event.data.schemaId, schemaId, 'schema ids should be equal');
            assert.notEqual(event.data.blockNumber, undefined, 'should have a block number');
            assert.equal(event.data.count.toNumber(), 1, "message count should be 1");
        }
    });
})

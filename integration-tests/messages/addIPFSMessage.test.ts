import "@frequency-chain/api-augment";
import { KeyringPair } from "@polkadot/keyring/types";
import { PARQUET_BROADCAST } from "../schemas/fixtures/parquetBroadcastSchemaType";
import assert from "assert";
import { createAndFundKeypair, devAccounts } from "../scaffolding/helpers";
import { ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { u16, u32 } from "@polkadot/types";
import { loadIpfs, getBases } from "./loadIPFS";
import { firstValueFrom } from "rxjs";
import { MessageResponse } from "@frequency-chain/api-augment/interfaces";

describe("Add Offchain Message", function () {
    this.timeout(5000); // Override default timeout of 500ms to allow for IPFS node startup

    let keys: KeyringPair;
    let schemaId: u16;
    let dummySchemaId: u16;
    let messageBlockNumber: u32;

    let ipfs_cid_64: string;
    let ipfs_cid_32: string;
    let ipfs_node: any;
    const ipfs_payload_data = "This is a test of Frequency.";
    const ipfs_payload_len = ipfs_payload_data.length + 1;
    let starting_block: number;

    before(async function () {
        starting_block = (await firstValueFrom(ExtrinsicHelper.api.rpc.chain.getHeader())).number.toNumber();
        ipfs_node = await loadIpfs();
        const { base64, base32 } = await getBases();
        const file = await ipfs_node.add({ path: 'integration_test.txt', content: ipfs_payload_data }, { cidVersion: 1, onlyHash: true });
        ipfs_cid_64 = file.cid.toString(base64);
        ipfs_cid_32 = file.cid.toString(base32);

        keys = await createAndFundKeypair();

        // Create a new MSA
        const createMsa = ExtrinsicHelper.createMsa(keys);
        await createMsa.fundAndSend();

        // Create a schema for IPFS
        const createSchema = ExtrinsicHelper.createSchema(keys, PARQUET_BROADCAST, "Parquet", "IPFS");
        const [event] = await createSchema.fundAndSend();
        if (event && createSchema.api.events.schemas.SchemaCreated.is(event)) {
            [, schemaId] = event.data;
        }

        // Create a dummy on-chain schema
        const createDummySchema = ExtrinsicHelper.createSchema(keys, { type: "record", name: "Dummy on-chain schema", fields: [] }, "AvroBinary", "OnChain");
        const [dummySchemaEvent] = await createDummySchema.fundAndSend();
        if (dummySchemaEvent && createDummySchema.api.events.schemas.SchemaCreated.is(dummySchemaEvent)) {
            [, dummySchemaId] = dummySchemaEvent.data;
        }
    })

    it('should fail if insufficient funds', async function () {
        await assert.rejects(ExtrinsicHelper.addIPFSMessage(keys, schemaId, ipfs_cid_64, ipfs_payload_len).signAndSend(), {
            message: /Inability to pay some fees/,
        });
    }).timeout(500);

    it('should fail if MSA is not valid (InvalidMessageSourceAccount)', async function () {
        const accountWithNoMsa = devAccounts[0].keys;
        await assert.rejects(ExtrinsicHelper.addIPFSMessage(accountWithNoMsa, schemaId, ipfs_cid_64, ipfs_payload_len).signAndSend(), {
            name: 'InvalidMessageSourceAccount',
            section: 'messages',
        });
    }).timeout(500);

    it('should fail if schema does not exist (InvalidSchemaId)', async function () {
        // Pick an arbitrarily high schemaId, such that it won't exist on the test chain.
        // If we ever create more than 999 schemas in a test suite/single Frequency instance, this test will fail.
        const f = ExtrinsicHelper.addIPFSMessage(keys, 999, ipfs_cid_64, ipfs_payload_len);
        await assert.rejects(f.fundAndSend(), {
            name: 'InvalidSchemaId',
            section: 'messages',
        });
    }).timeout(500);

    it("should fail if schema payload location is not IPFS (InvalidPayloadLocation)", async function () {
        const op = ExtrinsicHelper.addIPFSMessage(keys, dummySchemaId, ipfs_cid_64, ipfs_payload_len);
        await assert.rejects(op.fundAndSend(), { name: "InvalidPayloadLocation" });
    }).timeout(500);

    it("should fail if CID cannot be decoded (InvalidCid)", async function () {
        const f = ExtrinsicHelper.addIPFSMessage(keys, schemaId, "foo", ipfs_payload_len);
        await assert.rejects(f.fundAndSend(), { name: "InvalidCid" });
    }).timeout(500);

    it("should fail if CID is CIDv0 (UnsupportedCidVersion)", async function () {
        const file = await ipfs_node.add({ path: 'integration_test.txt', content: ipfs_payload_data }, { cidVersion: 0 });
        const cidV0 = file.cid.toString();
        const f = ExtrinsicHelper.addIPFSMessage(keys, schemaId, cidV0, ipfs_payload_len);
        await assert.rejects(f.fundAndSend(), { name: "UnsupportedCidVersion" });
    }).timeout(500);

    it("should successfully add an IPFS message", async function () {
        const f = ExtrinsicHelper.addIPFSMessage(keys, schemaId, ipfs_cid_64, ipfs_payload_len);
        const [event] = await f.fundAndSend();

        assert.notEqual(event, undefined, "should have returned a MessagesStored event");
        if (event && f.api.events.messages.MessagesStored.is(event)) {
            messageBlockNumber = event.data.blockNumber;
            assert.deepEqual(event.data.schemaId, schemaId, 'schema ids should be equal');
            assert.notEqual(event.data.blockNumber, undefined, 'should have a block number');
            assert.equal(event.data.count.toNumber(), 1, "message count should be 1");
        }
    });

    it("should successfully add onchain message", async function () {
        const f = ExtrinsicHelper.addOnChainMessage(keys, dummySchemaId, "0xdeadbeef");
        const [event] = await f.fundAndSend();

        assert.notEqual(event, undefined, "should have returned a MessagesStored event");
        if (event && f.api.events.messages.MessagesStored.is(event)) {
            messageBlockNumber = event.data.blockNumber;
            assert.deepEqual(event.data.schemaId, dummySchemaId, 'schema ids should be equal');
            assert.notEqual(event.data.blockNumber, undefined, 'should have a block number');
            assert.equal(event.data.count.toNumber(), 1, "message count should be 1");
        }
    });

    it("should successfully retrieve added message and returned CID should have Base32 encoding", async function () {
        const f = await firstValueFrom(ExtrinsicHelper.api.rpc.messages.getBySchemaId(schemaId, { from_block: starting_block, from_index: 0, to_block: starting_block + 999, page_size: 999 }));
        const response: MessageResponse = f.content[f.content.length - 1];
        const cid = Buffer.from(response.cid.unwrap()).toString();
        assert.equal(cid, ipfs_cid_32, 'returned CID should match base32-encoded CID');
    })
});

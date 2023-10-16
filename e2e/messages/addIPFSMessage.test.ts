import "@frequency-chain/api-augment";
import { KeyringPair } from "@polkadot/keyring/types";
import { base64 } from 'multiformats/bases/base64';
import { base32 } from 'multiformats/bases/base32';
import { CID } from 'multiformats/cid'
import { PARQUET_BROADCAST } from "../schemas/fixtures/parquetBroadcastSchemaType";
import assert from "assert";
import { createAndFundKeypair } from "../scaffolding/helpers";
import { ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { u16, u32 } from "@polkadot/types";
import { firstValueFrom } from "rxjs";
import { MessageResponse } from "@frequency-chain/api-augment/interfaces";
import { ipfsCid } from "./ipfs";
import { isDev } from "../scaffolding/env";
import { getFundingSource } from "../scaffolding/funding";

describe("Add Offchain Message", function () {
    const fundingSource = getFundingSource("messages-add-ipfs");

    let keys: KeyringPair;
    let schemaId: u16;
    let dummySchemaId: u16;
    let messageBlockNumber: u32;

    let ipfs_cid_64: string;
    let ipfs_cid_32: string;
    const ipfs_payload_data = "This is a test of Frequency.";
    const ipfs_payload_len = ipfs_payload_data.length + 1;
    let starting_block: number;

    before(async function () {
        starting_block = (await firstValueFrom(ExtrinsicHelper.api.rpc.chain.getHeader())).number.toNumber();

        const cid = await ipfsCid(ipfs_payload_data, './e2e_test.txt');
        ipfs_cid_64 = cid.toString(base64);
        ipfs_cid_32 = cid.toString(base32);

        keys = await createAndFundKeypair(fundingSource);

        // Create a new MSA
        const createMsa = ExtrinsicHelper.createMsa(keys);
        await createMsa.fundAndSend(fundingSource);

        // Create a schema for IPFS
        const createSchema = ExtrinsicHelper.createSchema(keys, PARQUET_BROADCAST, "Parquet", "IPFS");
        const [event] = await createSchema.fundAndSend(fundingSource);
        if (event && createSchema.api.events.schemas.SchemaCreated.is(event)) {
            [, schemaId] = event.data;
        }

        // Create a dummy on-chain schema
        const createDummySchema = ExtrinsicHelper.createSchema(keys, { type: "record", name: "Dummy on-chain schema", fields: [] }, "AvroBinary", "OnChain");
        const [dummySchemaEvent] = await createDummySchema.fundAndSend(fundingSource);
        if (dummySchemaEvent && createDummySchema.api.events.schemas.SchemaCreated.is(dummySchemaEvent)) {
            [, dummySchemaId] = dummySchemaEvent.data;
        }
    });

    it('should fail if insufficient funds', async function () {
        await assert.rejects(ExtrinsicHelper.addIPFSMessage(keys, schemaId, ipfs_cid_64, ipfs_payload_len).signAndSend(), {
            message: /Inability to pay some fees/,
        });
    });

    it('should fail if MSA is not valid (InvalidMessageSourceAccount)', async function () {
        const accountWithNoMsa = await createAndFundKeypair(fundingSource);
        await assert.rejects(ExtrinsicHelper.addIPFSMessage(accountWithNoMsa, schemaId, ipfs_cid_64, ipfs_payload_len).fundAndSend(fundingSource), {
            name: 'InvalidMessageSourceAccount',
            section: 'messages',
        });
    });

    it('should fail if schema does not exist (InvalidSchemaId)', async function () {
        // Pick an arbitrarily high schemaId, such that it won't exist on the test chain.
        // If we ever create more than 999 schemas in a test suite/single Frequency instance, this test will fail.
        const f = ExtrinsicHelper.addIPFSMessage(keys, 999, ipfs_cid_64, ipfs_payload_len);
        await assert.rejects(f.fundAndSend(fundingSource), {
            name: 'InvalidSchemaId',
            section: 'messages',
        });
    });

    it("should fail if schema payload location is not IPFS (InvalidPayloadLocation)", async function () {
        const op = ExtrinsicHelper.addIPFSMessage(keys, dummySchemaId, ipfs_cid_64, ipfs_payload_len);
        await assert.rejects(op.fundAndSend(fundingSource), { name: "InvalidPayloadLocation" });
    });

    it("should fail if CID cannot be decoded (InvalidCid)", async function () {
        const f = ExtrinsicHelper.addIPFSMessage(keys, schemaId, "foo", ipfs_payload_len);
        await assert.rejects(f.fundAndSend(fundingSource), { name: "InvalidCid" });
    });

    it("should fail if CID is CIDv0 (UnsupportedCidVersion)", async function () {
        const cid = await ipfsCid(ipfs_payload_data, './e2e_test.txt');
        const cidV0 = CID.createV0(cid.multihash as any).toString();
        const f = ExtrinsicHelper.addIPFSMessage(keys, schemaId, cidV0, ipfs_payload_len);
        await assert.rejects(f.fundAndSend(fundingSource), { name: "UnsupportedCidVersion" });
    });

    it("should successfully add an IPFS message", async function () {
        const f = ExtrinsicHelper.addIPFSMessage(keys, schemaId, ipfs_cid_64, ipfs_payload_len);
        const [event] = await f.fundAndSend(fundingSource);

        assert.notEqual(event, undefined, "should have returned a MessagesStored event");
        if (event && f.api.events.messages.MessagesStored.is(event)) {
            messageBlockNumber = event.data.blockNumber;
            assert.deepEqual(event.data.schemaId, schemaId, 'schema ids should be equal');
            assert.notEqual(event.data.blockNumber, undefined, 'should have a block number');
        }
    });

    it("should successfully retrieve added message and returned CID should have Base32 encoding", async function () {
        const f = await firstValueFrom(ExtrinsicHelper.api.rpc.messages.getBySchemaId(schemaId, { from_block: starting_block, from_index: 0, to_block: starting_block + 999, page_size: 999 }));
        const response: MessageResponse = f.content[f.content.length - 1];
        const cid = Buffer.from(response.cid.unwrap()).toString();
        assert.equal(cid, ipfs_cid_32, 'returned CID should match base32-encoded CID');
    })

    describe("Add OnChain Message and successfully retrieve it", function () {
        it("should successfully add and retrieve an onchain message", async function () {
            const f = ExtrinsicHelper.addOnChainMessage(keys, dummySchemaId, "0xdeadbeef");
            const [event] = await f.fundAndSend(fundingSource);

            assert.notEqual(event, undefined, "should have returned a MessagesStored event");
            if (event && f.api.events.messages.MessagesStored.is(event)) {
                messageBlockNumber = event.data.blockNumber;
                assert.deepEqual(event.data.schemaId, dummySchemaId, 'schema ids should be equal');
                assert.notEqual(event.data.blockNumber, undefined, 'should have a block number');
            }

            const get = await firstValueFrom(ExtrinsicHelper.api.rpc.messages.getBySchemaId(
                    dummySchemaId,
                    { from_block: starting_block,
                    from_index: 0,
                    to_block: starting_block + 999,
                    page_size: 999
                    }
                ));
            const response: MessageResponse = get.content[get.content.length - 1];
            assert.equal(response.payload, "0xdeadbeef", "payload should be 0xdeadbeef");
        });
    });
});

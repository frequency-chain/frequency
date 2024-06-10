import '@frequency-chain/api-augment';
import { KeyringPair } from '@polkadot/keyring/types';
import { base64 } from 'multiformats/bases/base64';
import { base32 } from 'multiformats/bases/base32';
import { CID } from 'multiformats/cid';
import { PARQUET_BROADCAST } from '../schemas/fixtures/parquetBroadcastSchemaType';
import assert from 'assert';
import { assertHasMessage, createAndFundKeypair, getOrCreateDummySchema } from '../scaffolding/helpers';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { u16 } from '@polkadot/types';
import { ipfsCid } from './ipfs';
import { getFundingSource } from '../scaffolding/funding';

const fundingSource = getFundingSource('messages-add-ipfs');
const ipfs_payload_data = 'This is a test of Frequency.';
const ipfs_payload_len = ipfs_payload_data.length + 1;

describe('Add Offchain Message', function () {
  let keys: KeyringPair;
  let schemaId: u16;
  let dummySchemaId: u16;

  let ipfs_cid_64: string;
  let ipfs_cid_32: string;
  let starting_block: number;

  before(async function () {
    starting_block = (await ExtrinsicHelper.apiPromise.rpc.chain.getHeader()).number.toNumber();

    const cid = await ipfsCid(ipfs_payload_data, './e2e_test.txt');
    ipfs_cid_64 = cid.toString(base64);
    ipfs_cid_32 = cid.toString(base32);

    keys = await createAndFundKeypair(fundingSource);

    // Create a new MSA
    const createMsa = ExtrinsicHelper.createMsa(keys);
    await createMsa.fundAndSend(fundingSource);

    // Create a schema for IPFS
    schemaId = (await ExtrinsicHelper.getOrCreateSchemaV3(
      fundingSource,
      PARQUET_BROADCAST,
      'Parquet',
      'IPFS',
      [],
      'test.addIPFSMessage'
    ))!;

    // Create a dummy on-chain schema
    dummySchemaId = await getOrCreateDummySchema(fundingSource);
  });

  it('should fail if insufficient funds', async function () {
    await assert.rejects(
      ExtrinsicHelper.addIPFSMessage(keys, schemaId, ipfs_cid_64, ipfs_payload_len).signAndSend('current'),
      {
        name: 'RpcError',
        message: /Inability to pay some fees/,
      }
    );
  });

  it('should fail if MSA is not valid (InvalidMessageSourceAccount)', async function () {
    const accountWithNoMsa = await createAndFundKeypair(fundingSource);
    await assert.rejects(
      ExtrinsicHelper.addIPFSMessage(accountWithNoMsa, schemaId, ipfs_cid_64, ipfs_payload_len).fundAndSend(
        fundingSource
      ),
      {
        name: 'InvalidMessageSourceAccount',
        section: 'messages',
      }
    );
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

  it('should fail if schema payload location is not IPFS (InvalidPayloadLocation)', async function () {
    const op = ExtrinsicHelper.addIPFSMessage(keys, dummySchemaId, ipfs_cid_64, ipfs_payload_len);
    await assert.rejects(op.fundAndSend(fundingSource), { name: 'InvalidPayloadLocation' });
  });

  it('should fail if CID cannot be decoded (InvalidCid)', async function () {
    const f = ExtrinsicHelper.addIPFSMessage(keys, schemaId, 'foo', ipfs_payload_len);
    await assert.rejects(f.fundAndSend(fundingSource), { name: 'InvalidCid' });
  });

  it('should fail if CID is CIDv0 (UnsupportedCidVersion)', async function () {
    const cid = await ipfsCid(ipfs_payload_data, './e2e_test.txt');
    const cidV0 = CID.createV0(cid.multihash as any).toString();
    const f = ExtrinsicHelper.addIPFSMessage(keys, schemaId, cidV0, ipfs_payload_len);
    await assert.rejects(f.fundAndSend(fundingSource), { name: 'UnsupportedCidVersion' });
  });

  it('should successfully add an IPFS message', async function () {
    const f = ExtrinsicHelper.addIPFSMessage(keys, schemaId, ipfs_cid_64, ipfs_payload_len);
    const { target: event } = await f.fundAndSend(fundingSource);

    assert.notEqual(event, undefined, 'should have returned a MessagesInBlock event');
  });

  it('should successfully retrieve added message and returned CID should have Base32 encoding', async function () {
    const f = await ExtrinsicHelper.apiPromise.rpc.messages.getBySchemaId(schemaId, {
      from_block: starting_block,
      from_index: 0,
      to_block: starting_block + 999,
      page_size: 999,
    });
    assertHasMessage(f, (x) => {
      const cid = x.cid.isSome && Buffer.from(x.cid.unwrap()).toString();
      return cid === ipfs_cid_32;
    });
  });

  describe('Add OnChain Message and successfully retrieve it', function () {
    it('should successfully add and retrieve an onchain message', async function () {
      const f = ExtrinsicHelper.addOnChainMessage(keys, dummySchemaId, '0xdeadbeef');
      const { target: event } = await f.fundAndSend(fundingSource);

      assert.notEqual(event, undefined, 'should have returned a MessagesInBlock event');

      const get = await ExtrinsicHelper.apiPromise.rpc.messages.getBySchemaId(dummySchemaId, {
        from_block: starting_block,
        from_index: 0,
        to_block: starting_block + 999,
        page_size: 999,
      });
      assertHasMessage(get, (x) => x.payload.isSome && x.payload.toString() === '0xdeadbeef');
    });
  });
});

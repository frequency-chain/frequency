import '@frequency-chain/api-augment';

import assert from 'assert';

import { KeyringPair } from '@polkadot/keyring/types';
import { merkleizeMetadata } from '@polkadot-api/merkleize-metadata';
import { Extrinsic, ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import {
  createKeys,
  createAndFundKeypair,
  assertExtrinsicSuccess,
  generateSchemaPartialName,
} from '../scaffolding/helpers';
import { getFundingSource } from '../scaffolding/funding';
import { u8aToHex } from '@polkadot/util';

const fundingSource = getFundingSource('check-metadata-hash');

// This is skipped as it requires the e2e tests to be run
// against a Frequency build that has the metadata-hash feature
// enabled. That feature is a large increase in compile time however.
describe.skip('Check Metadata Hash', function () {
  let keys: KeyringPair;
  let accountWithNoFunds: KeyringPair;

  before(async function () {
    keys = await createAndFundKeypair(fundingSource, 10_000_000n);
    accountWithNoFunds = createKeys();
  });

  it('should successfully transfer funds', async function () {
    const tx = ExtrinsicHelper.api.tx.balances.transferKeepAlive(accountWithNoFunds.address, 5_000_000n);

    const api = ExtrinsicHelper.apiPromise;
    const metadata = await api.call.metadata.metadataAtVersion(15);
    const { specName, specVersion } = api.runtimeVersion;
    const merkleInfo = {
      base58Prefix: api.consts.system.ss58Prefix.toNumber(),
      decimals: api.registry.chainDecimals[0],
      specName: specName.toString(),
      specVersion: specVersion.toNumber(),
      tokenSymbol: api.registry.chainTokens[0],
    };

    const merkleizedMetadata = merkleizeMetadata(metadata.toHex(), merkleInfo);
    const metadataHash = u8aToHex(merkleizedMetadata.digest());

    const extrinsic = new Extrinsic(() => tx, keys);

    const { eventMap } = await extrinsic.signAndSend('auto', {
      withSignedTransaction: true,
      mode: 1,
      metadataHash,
    });

    assertExtrinsicSuccess(eventMap);
  });
});

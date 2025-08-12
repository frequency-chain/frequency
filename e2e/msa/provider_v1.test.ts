import '@frequency-chain/api-augment';
import assert from 'assert';
import { createAndFundKeypair, DOLLARS, generateValidProviderPayloadWithName } from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';

let fundingSource: KeyringPair;

describe('Create Provider', function () {
  let keys: KeyringPair;
  let failureKeys: KeyringPair;

  before(async function () {
    fundingSource = await getFundingSource(import.meta.url);
    keys = await createAndFundKeypair(fundingSource, 5n * DOLLARS);
    failureKeys = await createAndFundKeypair(fundingSource, 5n * DOLLARS, 'failure-keys');
  });

  describe('createProvider', function () {
    it('should successfully create a provider', async function () {
      const f = ExtrinsicHelper.createMsa(keys);
      await f.fundAndSend(fundingSource);
      const createProviderOp = ExtrinsicHelper.createProvider(keys, 'MyProvider');
      const { target: providerEvent } = await createProviderOp.signAndSend();
      assert.notEqual(providerEvent, undefined, 'setup should return a ProviderCreated event');
      const providerId = providerEvent!.data.providerId;
      // assert providerId is greater than 0
      assert(providerId.toBigInt() > 0n, 'providerId should be greater than 0');
    });

    it('should fail to create a provider for long name', async function () {
      const f = ExtrinsicHelper.createMsa(failureKeys);
      await f.fundAndSend(fundingSource);
      const longName = 'a'.repeat(257); // 256 characters long limit
      const createProviderOp = ExtrinsicHelper.createProvider(failureKeys, longName);
      await assert.rejects(createProviderOp.signAndSend(), {
        name: 'ExceedsMaxProviderNameSize',
      });
    });
  });
});

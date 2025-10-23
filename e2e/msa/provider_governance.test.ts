import '@frequency-chain/api-augment';
import assert from 'assert';
import { createAndFundKeypair, DOLLARS, generateValidProviderPayloadWithName } from '../scaffolding/helpers';
import type { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { getFundingSource, getSudo } from '../scaffolding/funding';

let fundingSource: KeyringPair;

describe('Create Provider', function () {
  let keys: KeyringPair;
  let failureKeys: KeyringPair;
  let sudoKeys: KeyringPair;

  before(async function () {
    fundingSource = await getFundingSource(import.meta.url);
    sudoKeys = await getSudo().keys;
    keys = await createAndFundKeypair(fundingSource, 5n * DOLLARS);
    failureKeys = await createAndFundKeypair(fundingSource, 5n * DOLLARS, 'failure-keys');
  });

  it('should successfully create a provider', async function () {
    const f = ExtrinsicHelper.createMsa(keys);
    await f.fundAndSend(fundingSource);
    const createProviderOp = ExtrinsicHelper.createProviderViaGovernanceV2(sudoKeys, keys, {
      defaultName: 'MyProviderNew',
    });
    const { target: providerEvent } = await createProviderOp.sudoSignAndSend();
    assert.notEqual(providerEvent, undefined, 'setup should return a ProviderCreated event');
    const providerId = providerEvent!.data.providerId;
    // assert providerId is greater than 0
    assert(providerId.toBigInt() > 0n, 'providerId should be greater than 0');
  });

  it('should fail to create a provider for long name', async function () {
    const f = ExtrinsicHelper.createMsa(failureKeys);
    await f.fundAndSend(fundingSource);
    const longName = 'a'.repeat(257); // 256 characters long limit
    const createProviderOp = ExtrinsicHelper.createProviderViaGovernanceV2(keys, failureKeys, {
      defaultName: longName,
    });
    await assert.rejects(createProviderOp.signAndSend(), {
      name: 'RpcError',
    });
  });
});

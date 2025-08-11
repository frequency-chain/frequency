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
      const providerEntry = generateValidProviderPayloadWithName('MyProvider');
      const createProviderOp = ExtrinsicHelper.createProviderV2(keys, providerEntry);
      const { target: providerEvent } = await createProviderOp.signAndSend();
      assert.notEqual(providerEvent, undefined, 'setup should return a ProviderCreated event');
      const providerId = providerEvent!.data.providerId;
      // assert providerId is greater than 0
      assert(providerId.toBigInt() > 0n, 'providerId should be greater than 0');
    });

    it('should fail to create a provider for long name', async function () {
      const longName = 'a'.repeat(257); // 256 characters long limit
      const f = ExtrinsicHelper.createMsa(failureKeys);
      await f.fundAndSend(fundingSource);
      const providerEntry = generateValidProviderPayloadWithName(longName);

      const createProviderOp = ExtrinsicHelper.createProviderV2(failureKeys, providerEntry);
      await assert.rejects(createProviderOp.signAndSend(), undefined);
    });
  });

  it('should fail with invalid logo CID', async function () {
    const providerEntry = {
      defaultName: 'InvalidLogoProvider',
      defaultLogo250100PngCid: 'invalid-cid',
    };
    const createProviderOp = ExtrinsicHelper.createProviderV2(failureKeys, providerEntry);
    await assert.rejects(createProviderOp.signAndSend(), undefined);
  });

  it('should failt to create provider with wrong language code', async function () {
    const providerEntry = {
      defaultName: 'InvalidLanguageProvider',
      localizedNames: new Map([
        ['-xx', 'InvalidLanguageProvider'], // Invalid language code
        ['es&', 'ProveedorIdiomaInvalido'],
      ]),
    };
    const createProviderOp = ExtrinsicHelper.createProviderV2(failureKeys, providerEntry);
    await assert.rejects(createProviderOp.signAndSend(), undefined);
  });
});

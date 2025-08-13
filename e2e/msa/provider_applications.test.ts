import '@frequency-chain/api-augment';
import assert from 'assert';
import { createAndFundKeypair, DOLLARS, generateValidProviderPayloadWithName } from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { getFundingSource, getSudo } from '../scaffolding/funding';
import { isTestnet } from '../scaffolding/env';

let fundingSource: KeyringPair;

describe('Create Provider Application', function () {
  let keys: KeyringPair;
  let providerId: bigint;
  let sudoKeys: KeyringPair;

  before(async function () {
    sudoKeys = getSudo().keys;
    fundingSource = await getFundingSource(import.meta.url);
    keys = await createAndFundKeypair(fundingSource, 5n * DOLLARS);
    const f = ExtrinsicHelper.createMsa(keys);
    await f.fundAndSend(fundingSource);
    const providerEntry = generateValidProviderPayloadWithName('MyProvider');
    const createProviderOp = ExtrinsicHelper.createProviderV2(keys, providerEntry);
    const { target: providerEvent } = await createProviderOp.signAndSend();
    assert.notEqual(providerEvent, undefined, 'setup should return a ProviderCreated event');
    providerId = providerEvent!.data.providerId.toBigInt();
  });

  describe('create provider applications', function () {
    it('provider should exists', async function () {
      assert(providerId > 0n, 'providerId should be greater than 0');
    });

    it('should successfully create a provider application', async function () {
      if (isTestnet()) this.skip();
      const applicationEntry = generateValidProviderPayloadWithName('MyApp1');
      const createProviderOp = ExtrinsicHelper.createApplicationViaGovernance(sudoKeys, keys, applicationEntry);
      const { target: applicationEvent } = await createProviderOp.signAndSend();
      assert.notEqual(applicationEvent, undefined, 'setup should return a ProviderApplicationCreated event');
      const applicationId = applicationEvent!.data.applicationId;
      assert.equal(applicationId.toBigInt(), 0n, 'applicationId should be 0');
    });

    it('should fail to create a provider application for long name', async function () {
      if (isTestnet()) this.skip();
      const longName = 'a'.repeat(257); // 256 characters long limit
      const providerEntry = generateValidProviderPayloadWithName(longName);

      const createProviderOp = ExtrinsicHelper.createApplicationViaGovernance(sudoKeys, keys, providerEntry);
      await assert.rejects(createProviderOp.signAndSend(), {
        name: 'RpcError',
      });
    });
  });

  it('should fail with invalid logo CID', async function () {
    if (isTestnet()) this.skip();
    const applicationEntry = {
      defaultName: 'InvalidLogoProvider',
      defaultLogo250100PngCid: 'invalid-cid',
    };
    const createProviderOp = ExtrinsicHelper.createApplicationViaGovernance(sudoKeys, keys, applicationEntry);
    await assert.rejects(createProviderOp.signAndSend(), {
      name: 'InvalidCid',
    });
  });

  it('should fail to create provider application with wrong language code', async function () {
    if (isTestnet()) this.skip();
    const applicationEntry = {
      defaultName: 'InvalidLanguageProvider',
      localizedNames: new Map([
        ['-xx', 'InvalidLanguageProvider'], // Invalid language code
        ['es&', 'ProveedorIdiomaInvalido'],
      ]),
    };
    const createProviderOp = ExtrinsicHelper.createApplicationViaGovernance(sudoKeys, keys, applicationEntry);
    await assert.rejects(createProviderOp.signAndSend(), {
      name: 'InvalidBCP47LanguageCode',
    });
  });
});

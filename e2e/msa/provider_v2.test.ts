import '@frequency-chain/api-augment';
import assert from 'assert';
import {
  computeCid,
  createAndFundKeypair,
  DOLLARS,
  generateValidProviderPayloadWithName,
} from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';
import fs from 'fs';
import path from 'path';
import { Bytes } from '@polkadot/types';
import { fileURLToPath } from 'url';

let fundingSource: KeyringPair;

// reconstruct __dirname in ESM
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

describe('Create Provider', function () {
  let keys: KeyringPair;
  let logoKeys: KeyringPair;
  let failureKeys: KeyringPair;

  before(async function () {
    fundingSource = await getFundingSource(import.meta.url);
    keys = await createAndFundKeypair(fundingSource, 5n * DOLLARS);
    logoKeys = await createAndFundKeypair(fundingSource, 5n * DOLLARS, 'logo-keys');
    failureKeys = await createAndFundKeypair(fundingSource, 5n * DOLLARS, 'failure-keys');
    const f = ExtrinsicHelper.createMsa(failureKeys);
    await f.fundAndSend(fundingSource);
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

    it('should successfully create a provider with default logo', async function () {
      const f = ExtrinsicHelper.createMsa(logoKeys);
      await f.fundAndSend(fundingSource);
      const providerEntry = generateValidProviderPayloadWithName('MyProvider');
      const logoBytes = new Uint8Array(fs.readFileSync(path.join(__dirname, 'frequency.png')));
      const buf = Array.from(logoBytes);
      const logoCidStr = await computeCid(logoBytes);
      providerEntry.defaultLogo250100PngCid = logoCidStr;
      const createProviderOp = ExtrinsicHelper.createProviderV2(logoKeys, providerEntry);
      const { target: providerEvent } = await createProviderOp.signAndSend();
      assert.notEqual(providerEvent, undefined, 'setup should return a ProviderCreated event');
      const providerId = providerEvent!.data.providerId;
      // assert providerId is greater than 0
      assert(providerId.toBigInt() > 0n, 'providerId should be greater than 0');
      const encodedBytes = new Bytes(ExtrinsicHelper.api.registry, buf);
      const uploadLogoOp = ExtrinsicHelper.uploadLogo(logoKeys, logoCidStr, encodedBytes);
      await assert.doesNotReject(uploadLogoOp.signAndSend(), undefined);
      const providerContextDefault = await ExtrinsicHelper.apiPromise.rpc.msa.getProviderApplicationContext(
        providerId,
        null,
        null
      );
      assert.notEqual(providerContextDefault, undefined, 'should return a valid provider application context');
      assert.equal(providerContextDefault.isSome, true, 'provider context should be some');
      const resultingApplicationContext = providerContextDefault.unwrap();
      assert.equal(resultingApplicationContext.provider_id.toBigInt(), providerId, 'providerId should match');
      assert.equal(
        resultingApplicationContext.default_logo_250_100_png_bytes.length,
        encodedBytes.length,
        'logo byte length should match'
      );
    });

    it('should fail to create a provider for long name', async function () {
      const longName = 'a'.repeat(257); // 256 characters long limit
      const providerEntry = generateValidProviderPayloadWithName(longName);

      const createProviderOp = ExtrinsicHelper.createProviderV2(failureKeys, providerEntry);
      await assert.rejects(createProviderOp.signAndSend(), {
        name: 'RpcError',
      });
    });
  });

  it('should fail with invalid logo CID', async function () {
    const providerEntry = {
      defaultName: 'InvalidLogoProvider',
      defaultLogo250100PngCid: 'invalid-cid',
    };
    const createProviderOp = ExtrinsicHelper.createProviderV2(failureKeys, providerEntry);
    await assert.rejects(createProviderOp.signAndSend(), {
      name: 'InvalidCid',
    });
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
    await assert.rejects(createProviderOp.signAndSend(), {
      name: 'InvalidBCP47LanguageCode',
    });
  });
});

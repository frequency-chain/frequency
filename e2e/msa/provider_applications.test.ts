import '@frequency-chain/api-augment';
import assert from 'assert';
import {
  createAndFundKeypair,
  DOLLARS,
  generateValidProviderPayloadWithName,
  computeCid,
} from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { getFundingSource, getSudo } from '../scaffolding/funding';
import { isTestnet } from '../scaffolding/env';
import { Bytes } from '@polkadot/types';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

// reconstruct __dirname in ESM
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

let fundingSource: KeyringPair;

describe('Create Provider Application', function () {
  let keys: KeyringPair;
  let nonProviderKeys: KeyringPair;
  let providerId: bigint;
  let sudoKeys: KeyringPair;

  before(async function () {
    sudoKeys = getSudo().keys;
    fundingSource = await getFundingSource(import.meta.url);
    keys = await createAndFundKeypair(fundingSource, 5n * DOLLARS);
    nonProviderKeys = await createAndFundKeypair(fundingSource, 5n * DOLLARS);
    const f = ExtrinsicHelper.createMsa(keys);
    await f.fundAndSend(fundingSource);
    const f2 = ExtrinsicHelper.createMsa(nonProviderKeys);
    await f2.fundAndSend(fundingSource);
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
      assert.notEqual(applicationId.toBigInt(), undefined, 'applicationId should be defined');
    });

    it('should fail to create a provider application for non provider', async function () {
      if (isTestnet()) this.skip();
      const applicationEntry = generateValidProviderPayloadWithName('MyApp1');
      const createProviderOp = ExtrinsicHelper.createApplicationViaGovernance(
        sudoKeys,
        nonProviderKeys,
        applicationEntry
      );
      await assert.rejects(createProviderOp.signAndSend(), {
        name: 'ProviderNotRegistered',
      });
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

  it('should successfully create a provider application and upload logo', async function () {
    if (isTestnet()) this.skip();
    // Create fake logo bytes, 130 bytes long
    const logoBytes = new Uint8Array(10);
    for (let i = 0; i < logoBytes.length; i++) logoBytes[i] = i % 256;
    const buf = Array.from(logoBytes);
    const applicationEntry = generateValidProviderPayloadWithName('lOgoProvider');
    const logoCidStr = await computeCid(logoBytes);
    applicationEntry.defaultLogo250100PngCid = logoCidStr;
    const createProviderOp = ExtrinsicHelper.createApplicationViaGovernance(sudoKeys, keys, applicationEntry);
    const { target: applicationEvent } = await createProviderOp.signAndSend();
    assert.notEqual(applicationEvent, undefined, 'setup should return a ProviderApplicationCreated event');
    const encodedBytes = new Bytes(ExtrinsicHelper.api.registry, buf);
    const uploadLogoOp = ExtrinsicHelper.uploadLogo(keys, logoCidStr, encodedBytes);
    await assert.doesNotReject(uploadLogoOp.signAndSend(), undefined);
  });

  it('should successfully upload logo and compute same CIDv1', async function () {
    if (isTestnet()) this.skip();
    // read frequency.png into logoBytes
    const logoBytes = new Uint8Array(fs.readFileSync(path.join(__dirname, 'frequency.png')));
    const buf = Array.from(logoBytes);
    const applicationEntry = generateValidProviderPayloadWithName('lOgoProvider');
    const logoCidStr = await computeCid(logoBytes);
    applicationEntry.defaultLogo250100PngCid = logoCidStr;
    const createProviderOp = ExtrinsicHelper.createApplicationViaGovernance(sudoKeys, keys, applicationEntry);
    const { target: applicationEvent } = await createProviderOp.signAndSend();
    assert.notEqual(applicationEvent, undefined, 'setup should return a ProviderApplicationCreated event');
    const encodedBytes = new Bytes(ExtrinsicHelper.api.registry, buf);
    const uploadLogoOp = ExtrinsicHelper.uploadLogo(keys, logoCidStr, encodedBytes);
    await assert.doesNotReject(uploadLogoOp.signAndSend(), undefined);
    const applicationId = applicationEvent!.data.applicationId;
    assert.notEqual(applicationId.toBigInt(), undefined, 'applicationId should be defined');
    const applicationContextDefault = await ExtrinsicHelper.apiPromise.rpc.msa.getProviderApplicationContext(
      providerId,
      applicationId,
      null
    );
    assert.notEqual(applicationContextDefault, undefined, 'should return a valid provider application context');
    assert.equal(applicationContextDefault.isSome, true, 'provider context should be some');
    const resultingApplicationContext = applicationContextDefault.unwrap();
    assert.equal(resultingApplicationContext.provider_id.toBigInt(), providerId, 'providerId should match');
    assert.equal(resultingApplicationContext.application_id, applicationId.toBigInt(), 'applicationId should match');
    assert.equal(
      resultingApplicationContext.default_logo_250_100_png_bytes.length,
      encodedBytes.length,
      'logo byte length should match'
    );
  });

  it('should fail with LogoCidNotApproved error when uploading logo with unapproved CID', async function () {
    if (isTestnet()) this.skip();
    // Create fake logo bytes, 130 bytes long
    const applicationEntry = generateValidProviderPayloadWithName('lOgoProvider');
    const logoBytesDifferent = new Uint8Array(fs.readFileSync(path.join(__dirname, 'provider_applications.test.ts')));
    const logoCidStr = await computeCid(logoBytesDifferent);
    const buf = Array.from(logoBytesDifferent);
    const createProviderOp = ExtrinsicHelper.createApplicationViaGovernance(sudoKeys, keys, applicationEntry);
    const { target: applicationEvent } = await createProviderOp.signAndSend();
    assert.notEqual(applicationEvent, undefined, 'setup should return a ProviderApplicationCreated event');
    const encodedBytes = new Bytes(ExtrinsicHelper.api.registry, buf);
    const uploadLogoOp = ExtrinsicHelper.uploadLogo(keys, logoCidStr, encodedBytes);
    await assert.rejects(uploadLogoOp.signAndSend(), { name: 'LogoCidNotApproved' });
  });

  it('should fail with InvalidLogoBytes error when uploading logo as Uint8Array', async function () {
    if (isTestnet()) this.skip();
    // Create fake logo bytes, 130 bytes long
    const logoBytes = new Uint8Array(11);
    for (let i = 0; i < logoBytes.length; i++) logoBytes[i] = i % 256;
    const applicationEntry = generateValidProviderPayloadWithName('lOgoProvider');
    const logoCidStr = await computeCid(logoBytes);
    applicationEntry.defaultLogo250100PngCid = logoCidStr;
    const createProviderOp = ExtrinsicHelper.createApplicationViaGovernance(sudoKeys, keys, applicationEntry);
    const { target: applicationEvent } = await createProviderOp.signAndSend();
    assert.notEqual(applicationEvent, undefined, 'setup should return a ProviderApplicationCreated event');

    const encodedBytes = new Bytes(ExtrinsicHelper.api.registry, logoBytes); // this should fail because logoBytes is not a valid input
    const uploadLogoOp = ExtrinsicHelper.uploadLogo(keys, logoCidStr, encodedBytes);
    await assert.rejects(uploadLogoOp.signAndSend(), { name: 'InvalidLogoBytes' });
  });
});

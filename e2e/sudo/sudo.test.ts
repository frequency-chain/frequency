// All the sudo required tests must be in one test for parallelization needs

import '@frequency-chain/api-augment';
import { IntentId, MessageSourceId, SchemaId } from '@frequency-chain/api-augment/interfaces';
import type { KeyringPair } from '@polkadot/keyring/types';
import assert from 'assert';
import { Extrinsic, ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { isTestnet } from '../scaffolding/env';
import { getSudo, getFundingSource } from '../scaffolding/funding';
import { AVRO_GRAPH_CHANGE } from '../schemas/fixtures/avroGraphChangeSchemaType';
import { Bytes, u16, u64 } from '@polkadot/types';
import {
  DOLLARS,
  createDelegatorAndDelegation,
  createProviderKeysAndId,
  getCurrentItemizedHash,
  generateSchemaPartialName,
  createKeys,
  createMsaAndProvider,
  getOrCreateIntentAndSchema,
  assertExtrinsicSuccess,
  generateValidProviderPayloadWithName,
  computeCid,
  createAndFundKeypair, suppressConsoleLogs, restoreConsoleLogs,
} from '../scaffolding/helpers';
import { AVRO_CHAT_MESSAGE } from '../stateful-pallet-storage/fixtures/itemizedSchemaType';
import { stakeToProvider } from '../scaffolding/helpers';
import { AnyNumber } from '@polkadot/types/types';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { afterEach, beforeEach } from 'mocha';

// reconstruct __dirname in ESM
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

describe('Sudo required', function () {
  const sudoKey: KeyringPair = getSudo().keys;
  let fundingSource: KeyringPair;

  before(async function () {
    if (isTestnet()) this.skip();
    fundingSource = await getFundingSource(import.meta.url);
  });

  describe('schema#setMaxSchemaModelBytes', function () {
    it('should fail to set the schema size because of lack of root authority (BadOrigin)', async function () {
      const operation = new Extrinsic(
        () => ExtrinsicHelper.api.tx.schemas.setMaxSchemaModelBytes(1000000),
        fundingSource
      );
      await assert.rejects(operation.signAndSend(), { name: 'BadOrigin' });
    });

    it('should fail to set max schema size > hard-coded limit (SchemaModelMaxBytesBoundedVecLimit)', async function () {
      const limit = ExtrinsicHelper.api.consts.schemas.schemaModelMaxBytesBoundedVecLimit.toBigInt();
      const op = new Extrinsic(() => ExtrinsicHelper.api.tx.schemas.setMaxSchemaModelBytes(limit + 1n), sudoKey);
      await assert.rejects(op.sudoSignAndSend(), { name: 'ExceedsMaxSchemaModelBytes' });
    });

    it('should successfully set the max schema size', async function () {
      const size = (await ExtrinsicHelper.apiPromise.query.schemas.governanceSchemaModelMaxBytes()).toBigInt();
      const op = new Extrinsic(() => ExtrinsicHelper.api.tx.schemas.setMaxSchemaModelBytes(size + 1n), sudoKey);
      await op.sudoSignAndSend();
      assert.equal(true, true, 'operation should not have thrown error');
      const newSize = (await ExtrinsicHelper.apiPromise.query.schemas.governanceSchemaModelMaxBytes()).toBigInt();
      assert.equal(size + 1n, newSize, 'new size should have been set');
    });
  });

  describe('schema-pallet ', function () {
    let intentId: AnyNumber;

    it('should create intent with name using createIntentWithSettingsGov', async function () {
      if (isTestnet()) this.skip();

      const intentName = 'e-e.sudo-' + generateSchemaPartialName(15);
      const createIntent = ExtrinsicHelper.createIntentWithSettingsGov(sudoKey, 'Itemized', ['AppendOnly'], intentName);
      const { target: event, eventMap } = await createIntent.sudoSignAndSend();
      assert.notEqual(event, undefined);
      intentId = event?.data.intentId || new u16(ExtrinsicHelper.api.registry, 0);
      assert.notEqual(intentId.toNumber(), 0);
      const intent_response = await ExtrinsicHelper.getIntent(intentId);
      assert(intent_response.isSome);
      const intent_response_value = intent_response.unwrap();
      assert.notEqual(intent_response_value.settings.length, 0);
    });

    it('should create schema with using createSchemaGovV3', async function () {
      if (isTestnet()) this.skip();
      const createSchema = ExtrinsicHelper.createSchemaGovV3(sudoKey, intentId, AVRO_GRAPH_CHANGE, 'AvroBinary');
      const { target: event, eventMap } = await createSchema.sudoSignAndSend();
      assert.notEqual(event, undefined);
      const schemaId: u16 = event?.data.schemaId || new u16(ExtrinsicHelper.api.registry, 0);
      assert.notEqual(schemaId.toNumber(), 0);
      const schema_response = await ExtrinsicHelper.getSchema(schemaId);
      assert(schema_response.isSome);
      const schema_response_value = schema_response.unwrap();
      const schema_settings = schema_response_value.settings;
      assert.notEqual(schema_settings.length, 0);
    });
  });

  describe('stateful-pallet-storage', function () {
    it('should fail to create non itemized intent with AppendOnly settings', async function () {
      if (isTestnet()) this.skip();

      const intentName = 'e-e.sudo-' + generateSchemaPartialName(15);
      const ex = ExtrinsicHelper.createIntentWithSettingsGov(sudoKey, 'Paginated', ['AppendOnly'], intentName);
      await assert.rejects(ex.sudoSignAndSend(), {
        name: 'InvalidSetting',
      });
    });

    it('should not fail to create itemized intent with AppendOnly settings', async function () {
      if (isTestnet()) this.skip();

      const intentName = 'e-e.sudo-' + generateSchemaPartialName(15);
      const createSchema = ExtrinsicHelper.createIntentWithSettingsGov(sudoKey, 'Itemized', ['AppendOnly'], intentName);
      const { target: event } = await createSchema.sudoSignAndSend();
      assert.notEqual(event, undefined);
      const itemizedIntentId: u16 = event?.data.intentId || new u16(ExtrinsicHelper.api.registry, 0);
      assert.notEqual(itemizedIntentId.toNumber(), 0);
      const intent_response = await ExtrinsicHelper.getIntent(itemizedIntentId);
      assert(intent_response.isSome);
      const intent_response_value = intent_response.unwrap();
      const intent_settings = intent_response_value.settings;
      assert.notEqual(intent_settings.length, 0);
    });
  });

  describe('📗 Stateful Pallet Storage AppendOnly Schemas', function () {
    // This requires schema creation abilities

    let itemizedIntentId: IntentId;
    let itemizedSchemaId: SchemaId;
    let msa_id: MessageSourceId;
    let providerId: MessageSourceId;
    let providerKeys: KeyringPair;

    before(async function () {
      const intentName = 'e-e.sudo-' + generateSchemaPartialName(15);
      // Create a provider for the MSA, the provider will be used to grant delegation
      [providerKeys, providerId] = await createProviderKeysAndId(fundingSource, 2n * DOLLARS);
      assert.notEqual(providerId, undefined, 'setup should populate providerId');
      assert.notEqual(providerKeys, undefined, 'setup should populate providerKeys');

      // Create a schema for Itemized PayloadLocation
      const createIntent = ExtrinsicHelper.createIntentWithSettingsGov(sudoKey, 'Itemized', ['AppendOnly'], intentName);
      const { target: intentCreateEvent, eventMap: intentEventMap } = await createIntent.sudoSignAndSend();
      assertExtrinsicSuccess(intentEventMap);
      assert.notEqual(intentCreateEvent, undefined, 'setup should create intent');
      itemizedIntentId = intentCreateEvent!.data.intentId;

      const createSchema = ExtrinsicHelper.createSchemaGovV3(
        sudoKey,
        itemizedIntentId,
        AVRO_CHAT_MESSAGE,
        'AvroBinary'
      );
      const { target: schemaCreateEvent, eventMap: schemaEventMap } = await createSchema.sudoSignAndSend();
      assertExtrinsicSuccess(schemaEventMap);
      assert.notEqual(schemaCreateEvent, undefined, 'setup should create schema');
      itemizedSchemaId = schemaCreateEvent!.data.schemaId;

      // Create a MSA for the delegator and delegate to the provider for the itemized schema
      [, msa_id] = await createDelegatorAndDelegation(fundingSource, itemizedIntentId, providerId, providerKeys);
      assert.notEqual(msa_id, undefined, 'setup should populate msa_id');
    });

    describe('Itemized With AppendOnly Storage Tests', function () {
      it('should not be able to call delete action', async function () {
        // Add and update actions
        const payload_1 = new Bytes(ExtrinsicHelper.api.registry, 'Hello World From Frequency');

        const add_action = {
          Add: payload_1,
        };

        const payload_2 = new Bytes(ExtrinsicHelper.api.registry, 'Hello World Again From Frequency');

        const update_action = {
          Add: payload_2,
        };

        const idx_1: u16 = new u16(ExtrinsicHelper.api.registry, 1);

        const delete_action = {
          Delete: idx_1,
        };
        const target_hash = await getCurrentItemizedHash(msa_id, itemizedIntentId);

        const add_actions = [add_action, update_action, delete_action];

        const itemized_add_result_1 = ExtrinsicHelper.applyItemActions(
          providerKeys,
          itemizedSchemaId,
          msa_id,
          add_actions,
          target_hash
        );
        await assert.rejects(itemized_add_result_1.fundAndSend(fundingSource), {
          name: 'UnsupportedOperationForIntent',
          section: 'statefulStorage',
        });
      });
    });
  });

  describe('Capacity staking is affected by a hold being slashed', function () {
    it('stake succeeds when overlapping tokens are on hold due to a proposal', async function () {
      const accountBalance: bigint = 122n * DOLLARS;
      const stakeBalance: bigint = 100n * DOLLARS;
      const spendBalance: bigint = 20n * DOLLARS;
      const proposalBond: bigint = 100n * DOLLARS;

      // Setup some keys and a provider for capacity staking
      const stakeKeys: KeyringPair = createKeys('StakeKeys');
      const stakeProviderId: u64 = await createMsaAndProvider(
        fundingSource,
        stakeKeys,
        'StakeProvider',
        accountBalance
      );

      // Create a treasury proposal which will result in a hold with minimum bond = 100 DOLLARS
      const proposalExt = ExtrinsicHelper.submitProposal(stakeKeys, spendBalance);
      const { target: proposalEvent } = await proposalExt.signAndSend();
      assert.notEqual(proposalEvent, undefined, 'should return a Proposal event');

      // Confirm that the tokens were reserved/hold in the stakeKeys account using the query API
      let stakedAcctInfo = await ExtrinsicHelper.getAccountInfo(stakeKeys);
      assert.equal(
        stakedAcctInfo.data.reserved,
        proposalBond,
        `expected ${proposalBond} reserved balance, got ${stakedAcctInfo.data.reserved}`
      );

      // Create a stake that will result in overlapping tokens being frozen
      // stake will allow the frozen balance to overlap with the reserved balance from the proposal bond
      await assert.doesNotReject(stakeToProvider(fundingSource, stakeKeys, stakeProviderId, stakeBalance));

      // Slash the provider
      const slashExt = ExtrinsicHelper.rejectProposal(sudoKey, proposalEvent?.data['proposalIndex']);
      const { target: slashEvent } = await slashExt.sudoSignAndSend();
      assert.notEqual(slashEvent, undefined, 'should return a Treasury event');

      // Confirm that the tokens were slashed from the stakeKeys account using the query API
      stakedAcctInfo = await ExtrinsicHelper.getAccountInfo(stakeKeys);
      assert.equal(
        stakedAcctInfo.data.reserved,
        0n,
        `expected 0 reserved balance, got ${stakedAcctInfo.data.reserved}`
      );
    });

    it('proposal fails when there is Capacity staking', async function () {
      const accountBalance: bigint = 122n * DOLLARS;
      const stakeBalance: bigint = 100n * DOLLARS;
      const spendBalance: bigint = 20n * DOLLARS;

      // Setup some keys and a provider for capacity staking
      const stakeKeys: KeyringPair = createKeys('StakeKeys');
      const stakeProviderId: u64 = await createMsaAndProvider(
        fundingSource,
        stakeKeys,
        'StakeProvider',
        accountBalance
      );
      // Create a stake that will result in overlapping tokens being frozen
      await assert.doesNotReject(stakeToProvider(fundingSource, stakeKeys, stakeProviderId, stakeBalance));
      // Create a treasury proposal which will result in a hold with minimum bond = 100 DOLLARS
      // The proposal should fail because the stakeKeys account doesn't have enough
      // transferable to cover the deposit.
      const proposalExt = ExtrinsicHelper.submitProposal(stakeKeys, spendBalance);
      await assert.rejects(proposalExt.signAndSend());
    });
  });

  describe('Create Provider', function () {
    let keys: KeyringPair;
    let failureKeys: KeyringPair;

    before(async function () {
      keys = await createAndFundKeypair(fundingSource, 5n * DOLLARS, 'create-prov-good-keys');
      failureKeys = await createAndFundKeypair(fundingSource, 5n * DOLLARS, 'create-prov-failure-keys');
    });

    it('should successfully create a provider', async function () {
      const f = ExtrinsicHelper.createMsa(keys);
      await f.fundAndSend(fundingSource);
      const createProviderOp = ExtrinsicHelper.createProviderViaGovernanceV2(sudoKey, keys, {
        defaultName: 'MyProviderNew',
      });
      const { target: providerEvent } = await createProviderOp.sudoSignAndSend();
      await ExtrinsicHelper.waitForFinalization();
      assert.notEqual(providerEvent, undefined, 'setup should return a ProviderCreated event');
      const providerId = providerEvent!.data.providerId;
      // assert providerId is greater than 0
      assert(providerId.toBigInt() > 0n, 'providerId should be greater than 0');
    });
  });

  describe('Create Provider Application', function () {
    let providerKeys: KeyringPair;
    let nonProviderKeys: KeyringPair;
    let providerId: bigint;

    before(async function () {
      if (isTestnet()) this.skip();
      providerKeys = await createAndFundKeypair(fundingSource, 5n * DOLLARS, 'app-provider-key');
      nonProviderKeys = await createAndFundKeypair(fundingSource, 5n * DOLLARS, 'non-app-provider-key');
      let createMsaOp = ExtrinsicHelper.createMsa(providerKeys);
      await createMsaOp.fundAndSend(fundingSource);
      createMsaOp = ExtrinsicHelper.createMsa(nonProviderKeys);
      await createMsaOp.fundAndSend(fundingSource);
    });

    it('provider should exists', async function () {
      const providerEntry = generateValidProviderPayloadWithName('MyProvider1');
      const createProviderOp = ExtrinsicHelper.createProviderViaGovernanceV2(sudoKey, providerKeys, providerEntry);
      const { target: providerEvent } = await createProviderOp.sudoSignAndSend();
      await ExtrinsicHelper.waitForFinalization();
      assert.notEqual(providerEvent, undefined, 'setup should return a ProviderCreated event');
      providerId = providerEvent!.data.providerId.toBigInt();
      assert(providerId > 0n, 'providerId should be greater than 0');
    });

    it('should successfully create a provider application', async function () {
      if (isTestnet()) this.skip();
      const applicationEntry = generateValidProviderPayloadWithName('MyApp1ication');
      const createProviderOp = ExtrinsicHelper.createApplicationViaGovernance(sudoKey, providerKeys, applicationEntry);
      const { target: applicationEvent } = await createProviderOp.sudoSignAndSend();
      await ExtrinsicHelper.waitForFinalization();
      assert.notEqual(applicationEvent, undefined, 'setup should return a ProviderApplicationCreated event');
      const applicationId = applicationEvent!.data.applicationId;
      assert.notEqual(applicationId.toBigInt(), undefined, 'applicationId should be defined');
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
      const createProviderOp = ExtrinsicHelper.createApplicationViaGovernance(sudoKey, providerKeys, applicationEntry);
      const { target: applicationEvent } = await createProviderOp.sudoSignAndSend();
      await ExtrinsicHelper.waitForFinalization();
      assert.notEqual(applicationEvent, undefined, 'setup should return a ProviderApplicationCreated event');
      const encodedBytes = new Bytes(ExtrinsicHelper.api.registry, buf);
      const uploadLogoOp = ExtrinsicHelper.uploadLogo(providerKeys, logoCidStr, encodedBytes);
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
      const createProviderOp = ExtrinsicHelper.createApplicationViaGovernance(sudoKey, providerKeys, applicationEntry);
      const { target: applicationEvent } = await createProviderOp.sudoSignAndSend();
      await ExtrinsicHelper.waitForFinalization();
      assert.notEqual(applicationEvent, undefined, 'setup should return a ProviderApplicationCreated event');
      const encodedBytes = new Bytes(ExtrinsicHelper.api.registry, buf);
      const uploadLogoOp = ExtrinsicHelper.uploadLogo(providerKeys, logoCidStr, encodedBytes);
      await assert.doesNotReject(uploadLogoOp.signAndSend(), undefined);
      const applicationId = applicationEvent!.data.applicationId;
      assert.notEqual(applicationId.toBigInt(), undefined, 'applicationId should be defined');
      const applicationContextDefault =
        await ExtrinsicHelper.apiPromise.call.msaRuntimeApi.getProviderApplicationContext(
          providerId,
          applicationId,
          null
        );
      assert.notEqual(applicationContextDefault, undefined, 'should return a valid provider application context');
      assert.equal(applicationContextDefault.isSome, true, 'provider context should be some');
      const resultingApplicationContext = applicationContextDefault.unwrap();
      assert.equal(resultingApplicationContext.providerId.toBigInt(), providerId, 'providerId should match');
      assert.equal(resultingApplicationContext.applicationId, applicationId.toBigInt(), 'applicationId should match');
      assert.equal(
        resultingApplicationContext.defaultLogo250100PngBytes.unwrap().length,
        encodedBytes.length,
        'logo byte length should match'
      );
      const defaultName = new TextDecoder().decode(resultingApplicationContext.defaultName.toU8a(true));
      assert.equal(defaultName, 'lOgoProvider', 'default name should match');
    });

    it('should successfully create application with locale and retrieve', async function () {
      if (isTestnet()) this.skip();
      // read frequency.png into logoBytes
      const logoBytes = new Uint8Array(fs.readFileSync(path.join(__dirname, '/sudo.test.ts')));
      const buf = Array.from(logoBytes);
      const applicationEntry = generateValidProviderPayloadWithName('lOgoProvider');
      const logoCidStr = await computeCid(logoBytes);
      applicationEntry.defaultLogo250100PngCid = logoCidStr;
      const localizedNames = new Map([
        ['en', 'Logo Provider'],
        ['es', 'Proveedor de Logo'],
      ]);
      const localizedLogo = new Map([
        ['en', logoCidStr],
        ['es', logoCidStr],
      ]);
      applicationEntry.localizedNames = localizedNames;
      applicationEntry.localizedLogo250100PngCids = localizedLogo;
      const createProviderOp = ExtrinsicHelper.createApplicationViaGovernance(sudoKey, providerKeys, applicationEntry);
      const { target: applicationEvent } = await createProviderOp.sudoSignAndSend();
      await ExtrinsicHelper.waitForFinalization();
      assert.notEqual(applicationEvent, undefined, 'setup should return a ProviderApplicationCreated event');
      const encodedBytes = new Bytes(ExtrinsicHelper.api.registry, buf);
      const uploadLogoOp = ExtrinsicHelper.uploadLogo(providerKeys, logoCidStr, encodedBytes);
      await assert.doesNotReject(uploadLogoOp.signAndSend(), undefined);
      const applicationId = applicationEvent!.data.applicationId;
      assert.notEqual(applicationId.toBigInt(), undefined, 'applicationId should be defined');
      const applicationContextDefault =
        await ExtrinsicHelper.apiPromise.call.msaRuntimeApi.getProviderApplicationContext(
          providerId,
          applicationId,
          'en'
        );
      assert.notEqual(applicationContextDefault, undefined, 'should return a valid provider application context');
      assert.equal(applicationContextDefault.isSome, true, 'provider context should be some');
      const resultingApplicationContext = applicationContextDefault.unwrap();
      assert.equal(resultingApplicationContext.providerId.toBigInt(), providerId, 'providerId should match');
      assert.equal(resultingApplicationContext.applicationId, applicationId.toBigInt(), 'applicationId should match');
      assert.equal(
        resultingApplicationContext.defaultLogo250100PngBytes.unwrap().length,
        encodedBytes.length,
        'default logo byte length should match'
      );
      const localized_name_vec_u8 = resultingApplicationContext.localizedName;
      const localized_name_string = new TextDecoder().decode(localized_name_vec_u8.unwrap().toU8a(true));
      assert.equal(localized_name_string, 'Logo Provider', 'localized name (en) should match');
      assert.equal(
        resultingApplicationContext.localizedLogo250100PngBytes.unwrap().length,
        encodedBytes.length,
        'locale logo byte length should match'
      );
      const defaultName = new TextDecoder().decode(resultingApplicationContext.defaultName.toU8a(true));
      assert.equal(defaultName, 'lOgoProvider', 'default name should match');
    });

    it('should fail to create a provider application for non provider', async function () {
      if (isTestnet()) this.skip();
      const applicationEntry = generateValidProviderPayloadWithName('MyAppSomething');
      const createProviderOp = ExtrinsicHelper.createApplicationViaGovernance(
        sudoKey,
        nonProviderKeys,
        applicationEntry
      );
      await assert.rejects(createProviderOp.sudoSignAndSend(), {
        name: 'ProviderNotRegistered',
      });
    });

    it('should fail to create a provider application for long name', async function () {
      if (isTestnet()) this.skip();
      const longName = 'a'.repeat(257); // 256 characters long limit
      const providerEntry = generateValidProviderPayloadWithName(longName);
      const createProviderOp = ExtrinsicHelper.createApplicationViaGovernance(sudoKey, providerKeys, providerEntry);
      await assert.rejects(createProviderOp.sudoSignAndSend(), {
        name: 'RpcError',
      });
    });

    it('should fail with invalid logo CID', async function () {
      if (isTestnet()) this.skip();
      const applicationEntry = {
        defaultName: 'InvalidLogoProvider',
        defaultLogo250100PngCid: 'invalid-cid',
      };
      const createProviderOp = ExtrinsicHelper.createApplicationViaGovernance(sudoKey, providerKeys, applicationEntry);
      await assert.rejects(createProviderOp.sudoSignAndSend(), {
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
      const createProviderOp = ExtrinsicHelper.createApplicationViaGovernance(sudoKey, providerKeys, applicationEntry);
      await assert.rejects(createProviderOp.sudoSignAndSend(), {
        name: 'InvalidBCP47LanguageCode',
      });
    });

    it('should fail with InvalidLogoBytes error when uploading logo as Uint8Array', async function () {
      if (isTestnet()) this.skip();
      // Create fake logo bytes, 130 bytes long
      const logoBytes = new Uint8Array(11);
      for (let i = 0; i < logoBytes.length; i++) logoBytes[i] = i % 256;
      const applicationEntry = generateValidProviderPayloadWithName('lOgoProviderInvalid');
      const logoCidStr = await computeCid(logoBytes);
      applicationEntry.defaultLogo250100PngCid = logoCidStr;
      const createProviderOp = ExtrinsicHelper.createApplicationViaGovernance(sudoKey, providerKeys, applicationEntry);
      const { target: applicationEvent } = await createProviderOp.sudoSignAndSend();
      await ExtrinsicHelper.waitForFinalization();
      assert.notEqual(applicationEvent, undefined, 'setup should return a ProviderApplicationCreated event');

      const encodedBytes = new Bytes(ExtrinsicHelper.api.registry, logoBytes); // this should fail because logoBytes is not a valid input
      const uploadLogoOp = ExtrinsicHelper.uploadLogo(providerKeys, logoCidStr, encodedBytes);
      await assert.rejects(uploadLogoOp.signAndSend(), { name: 'InvalidLogoBytes' });
    });

    it('should fail with LogoCidNotApproved error when uploading logo with unapproved CID', async function () {
      if (isTestnet()) this.skip();
      // Create fake logo bytes, 130 bytes long
      const applicationEntry = generateValidProviderPayloadWithName('lOgoProviderFakeCid');
      const logoBytesDifferent = new Uint8Array(fs.readFileSync(path.join(__dirname, '../package.json')));
      const logoCidStr = await computeCid(logoBytesDifferent);
      const buf = Array.from(logoBytesDifferent);
      const createProviderOp = ExtrinsicHelper.createApplicationViaGovernance(sudoKey, providerKeys, applicationEntry);
      const { target: applicationEvent } = await createProviderOp.sudoSignAndSend();
      await ExtrinsicHelper.waitForFinalization();
      assert.notEqual(applicationEvent, undefined, 'setup should return a ProviderApplicationCreated event');
      const encodedBytes = new Bytes(ExtrinsicHelper.api.registry, buf);
      const uploadLogoOp = ExtrinsicHelper.uploadLogo(providerKeys, logoCidStr, encodedBytes);
      await assert.rejects(uploadLogoOp.signAndSend(), { name: 'LogoCidNotApproved' });
    });
  });

  describe('Update Provider and Application', function () {
    let providerKeys: KeyringPair;
    let providerId: bigint;
    let applicationId: bigint;

    before(async function () {
      if (isTestnet()) this.skip();
      fundingSource = await getFundingSource(import.meta.url);
      providerKeys = await createAndFundKeypair(fundingSource, 10n * DOLLARS, 'update-provider-key');

      // create MSA and register provider
      const f = ExtrinsicHelper.createMsa(providerKeys);
      await f.fundAndSend(fundingSource);

      const providerEntry = generateValidProviderPayloadWithName('UpdProv');
      const createProviderOp = ExtrinsicHelper.createProviderViaGovernanceV2(sudoKey, providerKeys, providerEntry);
      const { target: providerEvent } = await createProviderOp.sudoSignAndSend();
      assert.notEqual(providerEvent, undefined, 'should emit ProviderCreated');
      providerId = providerEvent!.data.providerId.toBigInt();
      assert(providerId > 0n, 'providerId should be > 0');
      // create a base application to update
      const app = generateValidProviderPayloadWithName('BaseApp');
      const createAppOp = ExtrinsicHelper.createApplicationViaGovernance(sudoKey, providerKeys, app);
      const { target: targetApp } = await createAppOp.sudoSignAndSend();
      await ExtrinsicHelper.waitForFinalization();
      assert.notEqual(targetApp, undefined, 'should emit ApplicationCreated');
      applicationId = targetApp!.data.applicationId.toBigInt();
      assert(applicationId >= 0n, 'applicationId should be > 0');
    });

    it('should successfully update provider defaultName via governance', async function () {
      if (isTestnet()) this.skip();
      const updated = generateValidProviderPayloadWithName('UpdProv2');
      const op = ExtrinsicHelper.updateProviderViaGovernance(sudoKey, providerKeys, updated);
      const { target } = await op.sudoSignAndSend();
      await ExtrinsicHelper.waitForFinalization();
      assert.notEqual(target, undefined, 'should emit ProviderUpdated');

      // Verify default name changed via runtime API
      const ctx = await ExtrinsicHelper.apiPromise.call.msaRuntimeApi.getProviderApplicationContext(
        providerId,
        null,
        null
      );
      assert.equal(ctx.isSome, true, 'provider context should be some');
      const result = ctx.unwrap();
      const defaultName = new TextDecoder().decode(result.defaultName.toU8a(true));
      assert.equal(defaultName, 'UpdProv2', 'provider default name should update');
    });

    it('should submit a proposal to update provider', async function () {
      if (isTestnet()) this.skip();
      const updated = generateValidProviderPayloadWithName('UpdProv3');
      const op = ExtrinsicHelper.proposeToUpdateProvider(providerKeys, updated);
      const { target } = await op.signAndSend();
      assert.notEqual(target, undefined, 'should emit Council.Proposed');
    });

    it('should update an existing application via governance', async function () {
      if (isTestnet()) this.skip();
      const updated = generateValidProviderPayloadWithName('AppUpdated');
      const op = ExtrinsicHelper.updateApplicationViaGovernance(sudoKey, providerKeys, applicationId, updated);
      const { target } = await op.sudoSignAndSend();
      await ExtrinsicHelper.waitForFinalization();
      assert.notEqual(target, undefined, 'should emit ApplicationContextUpdated');

      // fetch context and verify name
      const ctx = await ExtrinsicHelper.apiPromise.call.msaRuntimeApi.getProviderApplicationContext(
        providerId,
        applicationId,
        null
      );
      assert.equal(ctx.isSome, true, 'app context should be some');
      const result = ctx.unwrap();
      const defaultName = new TextDecoder().decode(result.defaultName.toU8a(true));
      assert.equal(defaultName, 'AppUpdated', 'app default name should update');
    });

    it('should submit a proposal to update application', async function () {
      if (isTestnet()) this.skip();
      // create an app to have an index
      const updated = generateValidProviderPayloadWithName('PropUpd');
      const op = ExtrinsicHelper.proposeToUpdateApplication(providerKeys, applicationId, updated);
      const { target: proposed } = await op.signAndSend();
      assert.notEqual(proposed, undefined, 'should emit Council.Proposed');
    });
  });
});

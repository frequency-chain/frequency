// All the sudo required tests must be in one test for parallelization needs

import '@frequency-chain/api-augment';
import { MessageSourceId, SchemaId } from '@frequency-chain/api-augment/interfaces';
import { KeyringPair } from '@polkadot/keyring/types';
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
  createAndFundKeypair,
  generateValidProviderPayloadWithName,
  computeCid,
} from '../scaffolding/helpers';
import { AVRO_CHAT_MESSAGE } from '../stateful-pallet-storage/fixtures/itemizedSchemaType';
import { stakeToProvider } from '../scaffolding/helpers';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

// reconstruct __dirname in ESM
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

describe('Sudo required', function () {
  let sudoKey: KeyringPair;
  let fundingSource: KeyringPair;
  let schemaName: string;

  before(async function () {
    if (isTestnet()) this.skip();
    sudoKey = getSudo().keys;
    fundingSource = await getFundingSource(import.meta.url);
    schemaName = 'e-e.sudo-' + generateSchemaPartialName(15);
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
    it('should create schema with name using createSchemaWithSettingsGovV2', async function () {
      if (isTestnet()) this.skip();
      const createSchema = ExtrinsicHelper.createSchemaWithSettingsGovV2(
        sudoKey,
        AVRO_GRAPH_CHANGE,
        'AvroBinary',
        'Itemized',
        'AppendOnly',
        schemaName
      );
      const { target: event, eventMap } = await createSchema.sudoSignAndSend();
      assert.notEqual(event, undefined);
      const itemizedSchemaId: u16 = event?.data.schemaId || new u16(ExtrinsicHelper.api.registry, 0);
      assert.notEqual(itemizedSchemaId.toNumber(), 0);
      const schema_response = await ExtrinsicHelper.getSchema(itemizedSchemaId);
      assert(schema_response.isSome);
      const schema_response_value = schema_response.unwrap();
      const schema_settings = schema_response_value.settings;
      assert.notEqual(schema_settings.length, 0);
      assert.notEqual(eventMap['schemas.SchemaNameCreated'], undefined);
    });
  });

  describe('stateful-pallet-storage', function () {
    it('should fail to create non itemized schema with AppendOnly settings', async function () {
      if (isTestnet()) this.skip();

      const ex = ExtrinsicHelper.createSchemaWithSettingsGovV2(
        sudoKey,
        AVRO_GRAPH_CHANGE,
        'AvroBinary',
        'Paginated',
        'AppendOnly',
        schemaName
      );
      await assert.rejects(ex.sudoSignAndSend(), {
        name: 'InvalidSetting',
      });
    });

    it('should not fail to create itemized schema with AppendOnly settings', async function () {
      if (isTestnet()) this.skip();

      const createSchema = ExtrinsicHelper.createSchemaWithSettingsGovV2(
        sudoKey,
        AVRO_GRAPH_CHANGE,
        'AvroBinary',
        'Itemized',
        'AppendOnly',
        schemaName
      );
      const { target: event } = await createSchema.sudoSignAndSend();
      assert.notEqual(event, undefined);
      const itemizedSchemaId: u16 = event?.data.schemaId || new u16(ExtrinsicHelper.api.registry, 0);
      assert.notEqual(itemizedSchemaId.toNumber(), 0);
      const schema_response = await ExtrinsicHelper.getSchema(itemizedSchemaId);
      assert(schema_response.isSome);
      const schema_response_value = schema_response.unwrap();
      const schema_settings = schema_response_value.settings;
      assert.notEqual(schema_settings.length, 0);
    });

    describe('📗 Stateful Pallet Storage AppendOnly Schemas', function () {
      // This requires schema creation abilities

      let itemizedSchemaId: SchemaId;
      let msa_id: MessageSourceId;
      let providerId: MessageSourceId;
      let providerKeys: KeyringPair;

      before(async function () {
        // Create a provider for the MSA, the provider will be used to grant delegation
        [providerKeys, providerId] = await createProviderKeysAndId(fundingSource, 2n * DOLLARS);
        assert.notEqual(providerId, undefined, 'setup should populate providerId');
        assert.notEqual(providerKeys, undefined, 'setup should populate providerKeys');

        // Create a schema for Itemized PayloadLocation
        const createSchema = ExtrinsicHelper.createSchemaWithSettingsGovV2(
          sudoKey,
          AVRO_CHAT_MESSAGE,
          'AvroBinary',
          'Itemized',
          'AppendOnly',
          schemaName
        );
        const { target: event } = await createSchema.sudoSignAndSend();
        itemizedSchemaId = event!.data.schemaId;

        // Create a MSA for the delegator and delegate to the provider for the itemized schema
        [, msa_id] = await createDelegatorAndDelegation(fundingSource, itemizedSchemaId, providerId, providerKeys);
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
          const target_hash = await getCurrentItemizedHash(msa_id, itemizedSchemaId);

          const add_actions = [add_action, update_action, delete_action];

          const itemized_add_result_1 = ExtrinsicHelper.applyItemActions(
            providerKeys,
            itemizedSchemaId,
            msa_id,
            add_actions,
            target_hash
          );
          await assert.rejects(itemized_add_result_1.fundAndSend(fundingSource), {
            name: 'UnsupportedOperationForSchema',
            section: 'statefulStorage',
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
          // stake will allow only the balance not on hold to be staked
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
    });
  });

  describe('Create Provider', function () {
    let keys: KeyringPair;
    let failureKeys: KeyringPair;

    before(async function () {
      keys = await createAndFundKeypair(fundingSource, 5n * DOLLARS);
      failureKeys = await createAndFundKeypair(fundingSource, 5n * DOLLARS, 'failure-keys');
      // Make sure we are finalized removing before trying to retire
      await ExtrinsicHelper.waitForFinalization();
    });

    it('should successfully create a provider and should fail to create a provider for long name', async function () {
      const f = ExtrinsicHelper.createMsa(keys);
      await f.fundAndSend(fundingSource);
      const createProviderOp = ExtrinsicHelper.createProviderViaGovernanceV2(sudoKey, keys, {
        defaultName: 'MyProviderNew',
      });
      const { target: providerEvent } = await createProviderOp.sudoSignAndSend();
      assert.notEqual(providerEvent, undefined, 'setup should return a ProviderCreated event');
      const providerId = providerEvent!.data.providerId;
      // assert providerId is greater than 0
      assert(providerId.toBigInt() > 0n, 'providerId should be greater than 0');

      // should fail to create a provider for long name
      const f2 = ExtrinsicHelper.createMsa(failureKeys);
      await f2.fundAndSend(fundingSource);
      const longName = 'a'.repeat(257); // 256 characters long limit
      const createProviderOp2 = ExtrinsicHelper.createProviderViaGovernanceV2(sudoKey, failureKeys, {
        defaultName: longName,
      });
      await assert.rejects(createProviderOp2.sudoSignAndSend(), {
        name: 'RpcError',
      });
    });
  });

  describe('Update Provider and Application', function () {
    let providerKeys: KeyringPair;
    let providerId: bigint;
    let applicationId: bigint;

    before(async function () {
      if (isTestnet()) this.skip();
      providerKeys = await createAndFundKeypair(fundingSource, 10n * DOLLARS, 'upd-provider');

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
      assert.notEqual(targetApp, undefined, 'should emit ApplicationCreated');
      applicationId = targetApp!.data.applicationId.toBigInt();
      assert(applicationId >= 0n, 'applicationId should be > 0');
    });

    it('should successfully update provider defaultName via governance', async function () {
      if (isTestnet()) this.skip();
      const updated = generateValidProviderPayloadWithName('UpdProv2');
      const op = ExtrinsicHelper.updateProviderViaGovernance(sudoKey, providerKeys, updated);
      const { target } = await op.sudoSignAndSend();
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

      // should submit a proposal to update provider
      const updated2 = generateValidProviderPayloadWithName('UpdProv3');
      const op2 = ExtrinsicHelper.proposeToUpdateProvider(providerKeys, updated2);
      const { target: target2 } = await op2.signAndSend();
      assert.notEqual(target2, undefined, 'should emit Council.Proposed');

      // should update an existing application via governance
      const updated3 = generateValidProviderPayloadWithName('AppUpdated');
      const op3 = ExtrinsicHelper.updateApplicationViaGovernance(sudoKey, providerKeys, applicationId, updated3);
      const { target: target3 } = await op3.sudoSignAndSend();
      assert.notEqual(target3, undefined, 'should emit ApplicationContextUpdated');

      // fetch context and verify name
      const ctx3 = await ExtrinsicHelper.apiPromise.call.msaRuntimeApi.getProviderApplicationContext(
        providerId,
        applicationId,
        null
      );
      assert.equal(ctx3.isSome, true, 'app context should be some');
      const result3 = ctx3.unwrap();
      const defaultName3 = new TextDecoder().decode(result3.defaultName.toU8a(true));
      assert.equal(defaultName3, 'AppUpdated', 'app default name should update');

      // should submit a proposal to update application
      // create an app to have an index
      const updated4 = generateValidProviderPayloadWithName('PropUpd');
      const op4 = ExtrinsicHelper.proposeToUpdateApplication(providerKeys, applicationId, updated4);
      const { target: proposed } = await op4.signAndSend();
      assert.notEqual(proposed, undefined, 'should emit Council.Proposed');
    });
  });

  describe('Create Provider Application', function () {
    let keys: KeyringPair;
    let nonProviderKeys: KeyringPair;
    let providerId: bigint;

    before(async function () {
      if (isTestnet()) this.skip();
      keys = await createAndFundKeypair(fundingSource, 20n * DOLLARS);
      nonProviderKeys = await createAndFundKeypair(fundingSource, 20n * DOLLARS);
      let createMsaOp = ExtrinsicHelper.createMsa(keys);
      await createMsaOp.fundAndSend(fundingSource);
      createMsaOp = ExtrinsicHelper.createMsa(nonProviderKeys);
      await createMsaOp.fundAndSend(fundingSource);
      const providerEntry = generateValidProviderPayloadWithName('MyProvider1');
      const createProviderOp = ExtrinsicHelper.createProviderViaGovernanceV2(sudoKey, keys, providerEntry);
      const { target: providerEvent } = await createProviderOp.sudoSignAndSend();
      assert.notEqual(providerEvent, undefined, 'setup should return a ProviderCreated event');
      providerId = providerEvent!.data.providerId.toBigInt();
    });

    it('provider should exists', async function () {
      assert(providerId > 0n, 'providerId should be greater than 0');
    });

    it('should successfully create a provider application', async function () {
      if (isTestnet()) this.skip();
      const applicationEntry = generateValidProviderPayloadWithName('MyApp1ication');
      const createProviderOp = ExtrinsicHelper.createApplicationViaGovernance(sudoKey, keys, applicationEntry);
      const { target: applicationEvent } = await createProviderOp.sudoSignAndSend();
      assert.notEqual(applicationEvent, undefined, 'setup should return a ProviderApplicationCreated event');
      const applicationId = applicationEvent!.data.applicationId;
      assert.notEqual(applicationId.toBigInt(), undefined, 'applicationId should be defined');
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
      const createProviderOp = ExtrinsicHelper.createApplicationViaGovernance(sudoKey, keys, providerEntry);
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
      const createProviderOp = ExtrinsicHelper.createApplicationViaGovernance(sudoKey, keys, applicationEntry);
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
      const createProviderOp = ExtrinsicHelper.createApplicationViaGovernance(sudoKey, keys, applicationEntry);
      await assert.rejects(createProviderOp.sudoSignAndSend(), {
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
      const createProviderOp = ExtrinsicHelper.createApplicationViaGovernance(sudoKey, keys, applicationEntry);
      const { target: applicationEvent } = await createProviderOp.sudoSignAndSend();
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
      const createProviderOp = ExtrinsicHelper.createApplicationViaGovernance(sudoKey, keys, applicationEntry);
      const { target: applicationEvent } = await createProviderOp.sudoSignAndSend();
      assert.notEqual(applicationEvent, undefined, 'setup should return a ProviderApplicationCreated event');
      const encodedBytes = new Bytes(ExtrinsicHelper.api.registry, buf);
      const uploadLogoOp = ExtrinsicHelper.uploadLogo(keys, logoCidStr, encodedBytes);
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

    it('should fail with LogoCidNotApproved error when uploading logo with unapproved CID', async function () {
      if (isTestnet()) this.skip();
      // Create fake logo bytes, 130 bytes long
      const applicationEntry = generateValidProviderPayloadWithName('lOgoProvider');
      const logoBytes = new Uint8Array(fs.readFileSync(path.join(__dirname, 'frequency.png')));
      const extra = new Uint8Array([0xde, 0xad, 0xbe, 0xef]); // example extra bytes
      const logoBytesDifferent = new Uint8Array(logoBytes.length + extra.length);
      logoBytesDifferent.set(logoBytes, 0);
      logoBytesDifferent.set(extra, logoBytes.length);
      const logoCidStr = await computeCid(logoBytesDifferent);
      const buf = Array.from(logoBytesDifferent);
      const createProviderOp = ExtrinsicHelper.createApplicationViaGovernance(sudoKey, keys, applicationEntry);
      const { target: applicationEvent } = await createProviderOp.sudoSignAndSend();
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
      const applicationEntry = generateValidProviderPayloadWithName('lOgoProviderInvalid');
      const logoCidStr = await computeCid(logoBytes);
      applicationEntry.defaultLogo250100PngCid = logoCidStr;
      const createProviderOp = ExtrinsicHelper.createApplicationViaGovernance(sudoKey, keys, applicationEntry);
      const { target: applicationEvent } = await createProviderOp.sudoSignAndSend();
      assert.notEqual(applicationEvent, undefined, 'setup should return a ProviderApplicationCreated event');

      const encodedBytes = new Bytes(ExtrinsicHelper.api.registry, logoBytes); // this should fail because logoBytes is not a valid input
      const uploadLogoOp = ExtrinsicHelper.uploadLogo(keys, logoCidStr, encodedBytes);
      await assert.rejects(uploadLogoOp.signAndSend(), { name: 'InvalidLogoBytes' });
    });

    it('should successfully create application with locale and retrieve', async function () {
      if (isTestnet()) this.skip();
      const frequencyBytes = new Uint8Array(fs.readFileSync(path.join(__dirname, 'frequency.png')));
      const extra = new Uint8Array([0xde, 0xad, 0xbe, 0xef]); // example extra bytes
      const logoBytes = new Uint8Array(frequencyBytes.length + extra.length);
      logoBytes.set(frequencyBytes, 0);
      logoBytes.set(extra, frequencyBytes.length);
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
      const createProviderOp = ExtrinsicHelper.createApplicationViaGovernance(sudoKey, keys, applicationEntry);
      const { target: applicationEvent } = await createProviderOp.sudoSignAndSend();
      assert.notEqual(applicationEvent, undefined, 'setup should return a ProviderApplicationCreated event');
      const encodedBytes = new Bytes(ExtrinsicHelper.api.registry, buf);
      const uploadLogoOp = ExtrinsicHelper.uploadLogo(keys, logoCidStr, encodedBytes);
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
  });
});

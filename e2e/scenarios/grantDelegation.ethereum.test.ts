import '@frequency-chain/api-augment';
import { KeyringPair } from '@polkadot/keyring/types';
import { u16, u64 } from '@polkadot/types';
import assert from 'assert';
import { AddProviderPayload, Extrinsic, ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import {
  DOLLARS,
  createAndFundKeypair,
  createAndFundKeypairs,
  generateDelegationPayload,
  signPayload,
} from '../scaffolding/helpers';
import { SchemaId } from '@frequency-chain/api-augment/interfaces';
import { getFundingSource } from '../scaffolding/funding';

const fundingSource = getFundingSource('grant-delegation-ethereum');

describe('Delegation Scenario Tests Ethereum', function () {
  let keys: KeyringPair;
  let otherMsaKeys: KeyringPair;
  let thirdMsaKeys: KeyringPair;
  let noMsaKeys: KeyringPair;
  let providerKeys: KeyringPair;
  let otherProviderKeys: KeyringPair;
  let schemaId: u16;
  let schemaId2: SchemaId;
  let providerId: u64;
  let otherProviderId: u64;
  let msaId: u64;
  let otherMsaId: u64;
  let thirdMsaId: u64;

  before(async function () {
    // Fund all the different keys
    [noMsaKeys, keys, otherMsaKeys, thirdMsaKeys, providerKeys, otherProviderKeys] = await createAndFundKeypairs(
      fundingSource,
      ['noMsaKeys', 'keys', 'otherMsaKeys', 'thirdMsaKeys', 'providerKeys', 'otherProviderKeys'],
      1n * DOLLARS,
      'ethereum'
    );

    const { target: msaCreatedEvent1 } = await ExtrinsicHelper.createMsa(keys).signAndSend();
    msaId = msaCreatedEvent1!.data.msaId;

    const { target: msaCreatedEvent2 } = await ExtrinsicHelper.createMsa(otherMsaKeys).signAndSend();
    otherMsaId = msaCreatedEvent2!.data.msaId;

    const { target: msaCreatedEvent3 } = await ExtrinsicHelper.createMsa(thirdMsaKeys).signAndSend();
    thirdMsaId = msaCreatedEvent3!.data.msaId;

    let createProviderMsaOp = ExtrinsicHelper.createMsa(providerKeys);
    await createProviderMsaOp.signAndSend();
    let createProviderOp = ExtrinsicHelper.createProvider(providerKeys, 'MyPoster');
    let { target: providerEvent } = await createProviderOp.signAndSend();
    assert.notEqual(providerEvent, undefined, 'setup should return a ProviderCreated event');
    providerId = providerEvent!.data.providerId;

    createProviderMsaOp = ExtrinsicHelper.createMsa(otherProviderKeys);
    await createProviderMsaOp.signAndSend();
    createProviderOp = ExtrinsicHelper.createProvider(otherProviderKeys, 'MyPoster');
    ({ target: providerEvent } = await createProviderOp.signAndSend());
    assert.notEqual(providerEvent, undefined, 'setup should return a ProviderCreated event');
    otherProviderId = providerEvent!.data.providerId;

    const schema = {
      type: 'record',
      name: 'Post',
      fields: [
        { name: 'title', type: { name: 'Title', type: 'string' } },
        { name: 'content', type: { name: 'Content', type: 'string' } },
        { name: 'fromId', type: { name: 'DSNPId', type: 'fixed', size: 8 } },
        { name: 'objectId', type: 'DSNPId' },
      ],
    };

    schemaId = await ExtrinsicHelper.getOrCreateSchemaV3(
      keys,
      schema,
      'AvroBinary',
      'OnChain',
      [],
      'test.grantDelegation'
    );

    schemaId2 = await ExtrinsicHelper.getOrCreateSchemaV3(
      keys,
      schema,
      'AvroBinary',
      'OnChain',
      [],
      'test.grantDelegationSecond'
    );
  });

  describe('delegation grants for a Ethereum key', function () {
    it('should fail to grant delegation if payload not signed by delegator (AddProviderSignatureVerificationFailed)', async function () {
      const payload = await generateDelegationPayload({
        authorizedMsaId: providerId,
        schemaIds: [schemaId],
      });
      const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload);

      const grantDelegationOp = ExtrinsicHelper.grantDelegation(
        keys,
        providerKeys,
        signPayload(providerKeys, addProviderData),
        payload
      );
      await assert.rejects(grantDelegationOp.fundAndSend(fundingSource), {
        name: 'AddProviderSignatureVerificationFailed',
      });
    });

    it('should fail to grant delegation if ID in payload does not match origin (UnauthorizedDelegator)', async function () {
      const payload = await generateDelegationPayload({
        authorizedMsaId: otherMsaId,
        schemaIds: [schemaId],
      });
      const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload);

      const grantDelegationOp = ExtrinsicHelper.grantDelegation(
        keys,
        providerKeys,
        signPayload(keys, addProviderData),
        payload
      );
      await assert.rejects(grantDelegationOp.fundAndSend(fundingSource), { name: 'UnauthorizedDelegator' });
    });

    it('should grant a delegation to a provider', async function () {
      const payload = await generateDelegationPayload({
        authorizedMsaId: providerId,
        schemaIds: [schemaId],
      });
      const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload);

      const grantDelegationOp = ExtrinsicHelper.grantDelegation(
        keys,
        providerKeys,
        signPayload(keys, addProviderData),
        payload
      );
      const { target: grantDelegationEvent } = await grantDelegationOp.fundAndSend(fundingSource);
      assert.notEqual(grantDelegationEvent, undefined, 'should have returned DelegationGranted event');
      assert.deepEqual(grantDelegationEvent?.data.providerId, providerId, 'provider IDs should match');
      assert.deepEqual(grantDelegationEvent?.data.delegatorId, msaId, 'delegator IDs should match');
    });
  });

  describe('createSponsoredAccountWithDelegation', function () {
    let sponsorKeys: KeyringPair;
    let op: Extrinsic;
    let defaultPayload: AddProviderPayload;

    before(async function () {
      sponsorKeys = await createAndFundKeypair(fundingSource, 50_000_000n, undefined, undefined, 'ethereum');
      defaultPayload = {
        authorizedMsaId: providerId,
        schemaIds: [schemaId],
      };
    });

    it('should successfully create a delegated account', async function () {
      const payload = await generateDelegationPayload(defaultPayload);
      const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload);

      op = ExtrinsicHelper.createSponsoredAccountWithDelegation(
        sponsorKeys,
        providerKeys,
        signPayload(sponsorKeys, addProviderData),
        payload
      );
      const { target: event, eventMap } = await op.fundAndSend(fundingSource);
      assert.notEqual(event, undefined, 'should have returned MsaCreated event');
      assert.notEqual(eventMap['msa.DelegationGranted'], undefined, 'should have returned DelegationGranted event');
      await assert.rejects(
        op.fundAndSend(fundingSource),
        { name: 'SignatureAlreadySubmitted' },
        'should reject double submission'
      );
    });
  });
});

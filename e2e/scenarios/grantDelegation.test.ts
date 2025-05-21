import '@frequency-chain/api-augment';
import { KeyringPair } from '@polkadot/keyring/types';
import { u16, u64 } from '@polkadot/types';
import assert from 'assert';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import {
  DOLLARS,
  createAndFundKeypairs,
  createMsaAndProvider,
  generateDelegationPayload,
  signPayloadSr25519,
} from '../scaffolding/helpers';
import { SchemaId } from '@frequency-chain/api-augment/interfaces';
import { getFundingSource } from '../scaffolding/funding';

const fundingSource = getFundingSource(import.meta.url);

describe('Delegation Scenario Tests', function () {
  let keys: KeyringPair;
  let otherMsaKeys: KeyringPair;
  let thirdMsaKeys: KeyringPair;
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
    [keys, otherMsaKeys, thirdMsaKeys, providerKeys, otherProviderKeys] = await createAndFundKeypairs(
      fundingSource,
      ['keys', 'otherMsaKeys', 'thirdMsaKeys', 'providerKeys', 'otherProviderKeys'],
      2n * DOLLARS
    );

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

    let msaCreatedEvent1, msaCreatedEvent2, msaCreatedEvent3;
    [{ target: msaCreatedEvent1 }, { target: msaCreatedEvent2 }, { target: msaCreatedEvent3 }, schemaId, schemaId2] =
      await Promise.all([
        ExtrinsicHelper.createMsa(keys).fundAndSend(fundingSource),
        ExtrinsicHelper.createMsa(otherMsaKeys).fundAndSend(fundingSource),
        ExtrinsicHelper.createMsa(thirdMsaKeys).fundAndSend(fundingSource),
        ExtrinsicHelper.getOrCreateSchemaV3(fundingSource, schema, 'AvroBinary', 'OnChain', [], 'test.grantDelegation'),
        ExtrinsicHelper.getOrCreateSchemaV3(
          fundingSource,
          schema,
          'AvroBinary',
          'OnChain',
          [],
          'test.grantDelegationSecond'
        ),
      ]);
    msaId = msaCreatedEvent1!.data.msaId;
    otherMsaId = msaCreatedEvent2!.data.msaId;
    thirdMsaId = msaCreatedEvent3!.data.msaId;

    [providerId, otherProviderId] = await Promise.all([
      createMsaAndProvider(fundingSource, providerKeys, 'MyPoster'),
      createMsaAndProvider(fundingSource, otherProviderKeys, 'MyPoster2'),
    ]);
    assert.notEqual(providerId, undefined, 'setup should return a Provider Id for Provider 1');
    assert.notEqual(otherProviderId, undefined, 'setup should return a Provider Id for Provider 2');
    // Make sure we are finalized before all the tests
    await ExtrinsicHelper.waitForFinalization();
  });

  it('should fail to grant delegation if payload not signed by delegator (AddProviderSignatureVerificationFailed)', async function () {
    const payload = await generateDelegationPayload({
      authorizedMsaId: providerId,
      schemaIds: [schemaId],
    });
    const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload);

    const grantDelegationOp = ExtrinsicHelper.grantDelegation(
      keys,
      providerKeys,
      signPayloadSr25519(providerKeys, addProviderData),
      payload
    );
    await assert.rejects(grantDelegationOp.fundAndSend(fundingSource), {
      name: 'AddProviderSignatureVerificationFailed',
    });
  });

  it('should fail to delegate to self (InvalidSelfProvider)', async function () {
    const payload = await generateDelegationPayload({
      authorizedMsaId: providerId,
      schemaIds: [schemaId],
    });
    const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload);

    const grantDelegationOp = ExtrinsicHelper.grantDelegation(
      providerKeys,
      providerKeys,
      signPayloadSr25519(providerKeys, addProviderData),
      payload
    );
    await assert.rejects(grantDelegationOp.fundAndSend(fundingSource), { name: 'InvalidSelfProvider' });
  });

  it('should fail to grant delegation to an MSA that is not a registered provider (ProviderNotRegistered)', async function () {
    const payload = await generateDelegationPayload({
      authorizedMsaId: otherMsaId,
      schemaIds: [schemaId],
    });
    const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload);

    const grantDelegationOp = ExtrinsicHelper.grantDelegation(
      keys,
      otherMsaKeys,
      signPayloadSr25519(keys, addProviderData),
      payload
    );
    await assert.rejects(grantDelegationOp.fundAndSend(fundingSource), { name: 'ProviderNotRegistered' });
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
      signPayloadSr25519(keys, addProviderData),
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
      signPayloadSr25519(keys, addProviderData),
      payload
    );
    const { target: grantDelegationEvent } = await grantDelegationOp.fundAndSend(fundingSource);
    assert.notEqual(grantDelegationEvent, undefined, 'should have returned DelegationGranted event');
    assert.deepEqual(grantDelegationEvent?.data.providerId, providerId, 'provider IDs should match');
    assert.deepEqual(grantDelegationEvent?.data.delegatorId, msaId, 'delegator IDs should match');
  });

  it('initial granted schemas should be correct', async function () {
    const schemaGrants = await ExtrinsicHelper.apiPromise.rpc.msa.grantedSchemaIdsByMsaId(msaId, providerId);
    assert.equal(schemaGrants.isSome, true);
    const schemaIds = schemaGrants
      .unwrap()
      .filter((grant) => grant.revoked_at.toBigInt() === 0n)
      .map((grant) => grant.schema_id.toNumber());
    const expectedSchemaIds = [schemaId.toNumber()];
    assert.deepStrictEqual(schemaIds, expectedSchemaIds, 'granted schemas should equal initial set');
  });

  it('should grant additional schema permissions', async function () {
    const payload = await generateDelegationPayload({
      authorizedMsaId: providerId,
      schemaIds: [schemaId, schemaId2],
    });
    const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload);

    const grantDelegationOp = ExtrinsicHelper.grantDelegation(
      keys,
      providerKeys,
      signPayloadSr25519(keys, addProviderData),
      payload
    );
    const { target: grantDelegationEvent } = await grantDelegationOp.fundAndSend(fundingSource);
    assert.notEqual(grantDelegationEvent, undefined, 'should have returned DelegationGranted event');
    assert.deepEqual(grantDelegationEvent?.data.providerId, providerId, 'provider IDs should match');
    assert.deepEqual(grantDelegationEvent?.data.delegatorId, msaId, 'delegator IDs should match');
    const grants = await ExtrinsicHelper.apiPromise.rpc.msa.grantedSchemaIdsByMsaId(msaId, providerId);
    const grantedSchemaIds = grants
      .unwrap()
      .filter((grant) => grant.revoked_at.toBigInt() === 0n)
      .map((grant) => grant.schema_id.toNumber());
    const expectedSchemaIds = [schemaId.toNumber(), schemaId2.toNumber()];
    assert.deepStrictEqual(grantedSchemaIds, expectedSchemaIds);
  });

  it('should get all delegations and grants', async function () {
    const payload = await generateDelegationPayload({
      authorizedMsaId: providerId,
      schemaIds: [schemaId, schemaId2],
    });
    const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload);
    const grantDelegationOp = ExtrinsicHelper.grantDelegation(
      thirdMsaKeys,
      providerKeys,
      signPayloadSr25519(thirdMsaKeys, addProviderData),
      payload
    );
    const { target: grantDelegationEvent } = await grantDelegationOp.fundAndSend(fundingSource);
    assert.notEqual(grantDelegationEvent, undefined, 'should have returned DelegationGranted event');

    const payload2 = await generateDelegationPayload({
      authorizedMsaId: otherProviderId,
      schemaIds: [schemaId, schemaId2],
    });
    const addProviderData2 = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload2);
    const grantDelegationOp2 = ExtrinsicHelper.grantDelegation(
      thirdMsaKeys,
      otherProviderKeys,
      signPayloadSr25519(thirdMsaKeys, addProviderData2),
      payload2
    );
    const { target: grantDelegationEvent2 } = await grantDelegationOp2.fundAndSend(fundingSource);
    assert.notEqual(grantDelegationEvent2, undefined, 'should have returned DelegationGranted event');

    const grants = await ExtrinsicHelper.apiPromise.rpc.msa.getAllGrantedDelegationsByMsaId(thirdMsaId);
    assert.deepStrictEqual(grants.length, 2);
    const expectedProviderIds = [providerId.toNumber(), otherProviderId.toNumber()];
    assert(expectedProviderIds.indexOf(grants[0].provider_id.toNumber()) !== -1);
    assert(expectedProviderIds.indexOf(grants[1].provider_id.toNumber()) !== -1);
  });
});

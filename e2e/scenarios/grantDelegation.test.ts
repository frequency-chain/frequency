import '@frequency-chain/api-augment';
import type { KeyringPair } from '@polkadot/keyring/types';
import { u16, u64 } from '@polkadot/types';
import assert from 'assert';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import {
  DOLLARS,
  createAndFundKeypairs,
  createMsaAndProvider,
  generateDelegationPayload,
  signPayloadSr25519,
  getOrCreateIntentAndSchema,
  getOrCreateDelegationSchema,
} from '../scaffolding/helpers';
import { getFundingSource } from '../scaffolding/funding';

let fundingSource: KeyringPair;

describe('Delegation Scenario Tests', function () {
  let keys: KeyringPair;
  let otherMsaKeys: KeyringPair;
  let thirdMsaKeys: KeyringPair;
  let providerKeys: KeyringPair;
  let otherProviderKeys: KeyringPair;
  let intentId: u16;
  let intentId2: u16;
  let providerId: u64;
  let otherProviderId: u64;
  let msaId: u64;
  let otherMsaId: u64;
  let thirdMsaId: u64;

  before(async function () {
    fundingSource = await getFundingSource(import.meta.url);
    // Fund all the different keys
    [keys, otherMsaKeys, thirdMsaKeys, providerKeys, otherProviderKeys] = await createAndFundKeypairs(
      fundingSource,
      ['keys', 'otherMsaKeys', 'thirdMsaKeys', 'providerKeys', 'otherProviderKeys'],
      2n * DOLLARS
    );

    let msaCreatedEvent1: any, msaCreatedEvent2: any, msaCreatedEvent3: any;

    [
      /* eslint-disable prefer-const */
      { target: msaCreatedEvent1 },
      { target: msaCreatedEvent2 },
      { target: msaCreatedEvent3 },
      /* eslint-enable prefer-const */
      { intentId },
      { intentId: intentId2 },
    ] = await Promise.all([
      ExtrinsicHelper.createMsa(keys).fundAndSend(fundingSource),
      ExtrinsicHelper.createMsa(otherMsaKeys).fundAndSend(fundingSource),
      ExtrinsicHelper.createMsa(thirdMsaKeys).fundAndSend(fundingSource),
      getOrCreateDelegationSchema(fundingSource),
      getOrCreateDelegationSchema(fundingSource, undefined, 'test.grantDelegationSecond'),
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
      intentIds: [intentId],
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
      intentIds: [intentId],
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
      intentIds: [intentId],
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
      intentIds: [intentId],
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
      intentIds: [intentId],
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

  it('initial granted intents should be correct via RPC (deprecated)', async function () {
    const intentGrants = await ExtrinsicHelper.apiPromise.rpc.msa.grantedSchemaIdsByMsaId(msaId, providerId);
    assert.equal(intentGrants.isSome, true);
    const intentIds = intentGrants
      .unwrap()
      .filter((grant) => grant.revoked_at.toBigInt() === 0n)
      .map((grant) => grant.schema_id.toNumber());
    const expectedIntentIds = [intentId.toNumber()];
    assert.deepStrictEqual(intentIds, expectedIntentIds, 'granted intents should equal initial set');
  });

  it('initial granted intents should be correct via Runtime API', async function () {
    const response = await ExtrinsicHelper.apiPromise.call.msaRuntimeApi.getDelegationForMsaAndProvider(
      msaId,
      providerId
    );
    assert.equal(response.isSome, true, 'response should be valid');
    const delegation = response.unwrap();
    console.log('Delegation is: ', delegation.toJSON());
    assert.equal(delegation.revokedAt.toNumber(), 0, 'delegation should not be revoked');
    const intentIds = delegation.permissions
      .filter((grant) => grant.revokedAt.toBigInt() === 0n)
      .map((grant) => grant.grantedId.toNumber());
    const expectedIntentIds = [intentId.toNumber()];
    assert.deepStrictEqual(intentIds, expectedIntentIds, 'granted intents should equal initial set');
  });

  it('should grant additional intent permissions', async function () {
    const payload = await generateDelegationPayload({
      authorizedMsaId: providerId,
      intentIds: [intentId, intentId2],
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
    const grantedIntentIds = grants
      .unwrap()
      .filter((grant) => grant.revoked_at.toBigInt() === 0n)
      .map((grant) => grant.schema_id.toNumber());
    const expectedIntentIds = [intentId.toNumber(), intentId2.toNumber()];
    // Sort them as order doesn't matter
    assert.deepStrictEqual(grantedIntentIds.sort(), expectedIntentIds.sort());
  });

  describe('multiple provider grants for an MSA', function () {
    before(async function () {
      const payload = await generateDelegationPayload({
        authorizedMsaId: providerId,
        intentIds: [intentId, intentId2],
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
        intentIds: [intentId, intentId2],
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
    });

    it('should get all delegations and grants via RPC (deprecated)', async function () {
      const grants = await ExtrinsicHelper.apiPromise.rpc.msa.getAllGrantedDelegationsByMsaId(thirdMsaId);
      assert.deepStrictEqual(grants.length, 2);
      const expectedProviderIds = [providerId.toNumber(), otherProviderId.toNumber()];
      assert(expectedProviderIds.indexOf(grants[0].provider_id.toNumber()) !== -1);
      assert(expectedProviderIds.indexOf(grants[1].provider_id.toNumber()) !== -1);
    });

    it('should get all delegations and grants via Runtime API', async function () {
      const grants = await ExtrinsicHelper.apiPromise.call.msaRuntimeApi.getAllGrantedDelegationsByMsaId(thirdMsaId);
      assert.deepStrictEqual(grants.length, 2);
      const expectedProviderIds = [providerId.toNumber(), otherProviderId.toNumber()];
      assert(expectedProviderIds.indexOf(grants[0].providerId.toNumber()) !== -1);
      assert(expectedProviderIds.indexOf(grants[1].providerId.toNumber()) !== -1);
    });
  });
});

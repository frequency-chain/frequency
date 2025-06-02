import '@frequency-chain/api-augment';
import { KeyringPair } from '@polkadot/keyring/types';
import { u16, u64 } from '@polkadot/types';
import assert from 'assert';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import {
  DOLLARS,
  createAndFundKeypair,
  createAndFundKeypairs,
  createKeys,
  createMsaAndProvider,
  generateDelegationPayload,
  getBlockNumber,
  signPayloadSr25519,
} from '../scaffolding/helpers';
import { SchemaGrantResponse } from '@frequency-chain/api-augment/interfaces';
import { getFundingSource } from '../scaffolding/funding';

const fundingSource = getFundingSource(import.meta.url);

describe('Delegation Scenario Tests: Revocation', function () {
  let keys: KeyringPair;
  let revokeKeys: KeyringPair;
  let providerKeys: KeyringPair;
  let otherProviderKeys: KeyringPair;
  let schemaId: u16;
  let providerId: u64;
  let otherProviderId: u64;
  let msaId: u64;

  before(async function () {
    // Fund all the different keys
    [keys, revokeKeys, providerKeys, otherProviderKeys] = await createAndFundKeypairs(
      fundingSource,
      ['keys', 'revokeKeys', 'providerKeys', 'otherProviderKeys'],
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

    let msaCreatedEvent1;
    [{ target: msaCreatedEvent1 }, schemaId] = await Promise.all([
      ExtrinsicHelper.createMsa(keys).fundAndSend(fundingSource),
      ExtrinsicHelper.getOrCreateSchemaV3(fundingSource, schema, 'AvroBinary', 'OnChain', [], 'test.grantDelegation'),
    ]);
    msaId = msaCreatedEvent1!.data.msaId;

    [providerId, otherProviderId] = await Promise.all([
      createMsaAndProvider(fundingSource, providerKeys, 'MyPoster'),
      createMsaAndProvider(fundingSource, otherProviderKeys, 'MyPoster2'),
    ]);
    assert.notEqual(providerId, undefined, 'setup should return a Provider Id for Provider 1');
    assert.notEqual(otherProviderId, undefined, 'setup should return a Provider Id for Provider 2');

    // Make sure we are finalized before all the tests
    await ExtrinsicHelper.waitForFinalization();
  });

  it('should fail to revoke a delegation if no MSA exists (InvalidMsaKey)', async function () {
    const nonMsaKeys = await createAndFundKeypair(fundingSource);
    const op = ExtrinsicHelper.revokeDelegationByDelegator(nonMsaKeys, providerId);
    await assert.rejects(op.signAndSend('current'), { name: 'RpcError', message: /Custom error: 1$/ });
  });

  describe('revocation path', function () {
    before(async function () {
      // Create Delegation
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
      const { target: grantDelegationEvent } = await grantDelegationOp.fundAndSend(fundingSource, false);
      assert.notEqual(grantDelegationEvent, undefined, 'should have returned DelegationGranted event');
    });

    it('should revoke a delegation by delegator', async function () {
      // Revoke Delegation
      const revokeDelegationOp = ExtrinsicHelper.revokeDelegationByDelegator(keys, providerId);
      const { target: revokeDelegationEvent } = await revokeDelegationOp.signAndSend();
      assert.notEqual(revokeDelegationEvent, undefined, 'should have returned DelegationRevoked event');
      assert.deepEqual(revokeDelegationEvent?.data.providerId, providerId, 'provider ids should be equal');
      assert.deepEqual(revokeDelegationEvent?.data.delegatorId, msaId, 'delegator ids should be equal');
      const delegation = await ExtrinsicHelper.apiPromise.query.msa.delegatorAndProviderToDelegation(msaId, providerId);
      assert(delegation.isSome);
      assert.notEqual(delegation.unwrap().revokedAt.toNumber(), 0, 'delegation revokedAt should not be zero');
    });

    it('should fail to revoke a delegation that has already been revoked (InvalidDelegation)', async function () {
      const op = ExtrinsicHelper.revokeDelegationByDelegator(keys, providerId);
      await assert.rejects(op.signAndSend('current'), { name: 'RpcError', message: /Custom error: 0$/ });
    });

    it('should fail to revoke delegation where no delegation exists (DelegationNotFound)', async function () {
      const op = ExtrinsicHelper.revokeDelegationByDelegator(keys, otherProviderId);
      await assert.rejects(op.signAndSend('current'), { name: 'RpcError', message: /Custom error: 0$/ });
    });
  });

  describe('Successful revocation', function () {
    let revokeMsaId: u64 | undefined;
    let revokedAtBlock: number;

    before(async function () {
      const payload = await generateDelegationPayload({ authorizedMsaId: providerId, schemaIds: [schemaId] });
      const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload);
      const op = ExtrinsicHelper.createSponsoredAccountWithDelegation(
        revokeKeys,
        providerKeys,
        signPayloadSr25519(revokeKeys, addProviderData),
        payload
      );
      const { target: msaEvent } = await op.fundAndSend(fundingSource, false);
      revokeMsaId = msaEvent?.data.msaId;
      assert.notEqual(revokeMsaId, undefined, 'should have returned an MSA');
    });

    it('schema permissions revoked block of delegation should be zero', async function () {
      const delegationsResponse = await ExtrinsicHelper.apiPromise.rpc.msa.grantedSchemaIdsByMsaId(
        revokeMsaId,
        providerId
      );
      assert(delegationsResponse.isSome);
      const delegations: SchemaGrantResponse[] = delegationsResponse.unwrap().toArray();
      delegations.forEach((delegation) => {
        assert(delegation.revoked_at.toBigInt() == 0n);
      });
    });

    it('should revoke a delegation by provider', async function () {
      const op = ExtrinsicHelper.revokeDelegationByProvider(revokeMsaId as u64, providerKeys);
      const { target: revokeEvent } = await op.signAndSend();
      assert.notEqual(revokeEvent, undefined, 'should have returned a DelegationRevoked event');
      assert.deepEqual(revokeEvent?.data.delegatorId, revokeMsaId, 'delegator ids should match');
      assert.deepEqual(revokeEvent?.data.providerId, providerId, 'provider ids should match');
      revokedAtBlock = await getBlockNumber();
    });

    it('revoked delegation should be reflected in all previously-granted schema permissions', async function () {
      // Make a block first to make sure the state has rolled to the next block
      const currentBlock = await getBlockNumber();
      ExtrinsicHelper.runToBlock(currentBlock + 1);
      const delegationsResponse = await ExtrinsicHelper.apiPromise.rpc.msa.grantedSchemaIdsByMsaId(
        revokeMsaId,
        providerId
      );
      assert(delegationsResponse.isSome);
      const delegations: SchemaGrantResponse[] = delegationsResponse.unwrap().toArray();
      delegations.forEach((delegation) => {
        const diff = delegation.revoked_at.toNumber() - revokedAtBlock;
        // Due to parallelization, this could be off by a few blocks
        assert(Math.abs(Number(diff.toString())) < 20);
      });
    });

    it('should re-grant a previously revoked delegation', async function () {
      const delegatorKeys = createKeys();
      const payload = await generateDelegationPayload({ authorizedMsaId: providerId, schemaIds: [schemaId] });
      const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload);
      const op = ExtrinsicHelper.createSponsoredAccountWithDelegation(
        delegatorKeys,
        providerKeys,
        signPayloadSr25519(delegatorKeys, addProviderData),
        payload
      );

      const { target: msaEvent } = await op.fundAndSend(fundingSource, false);
      const newMsaId = msaEvent?.data.msaId;
      assert.notEqual(newMsaId, undefined, 'should have returned an MSA');
      await assert.doesNotReject(
        ExtrinsicHelper.revokeDelegationByProvider(newMsaId!, providerKeys).signAndSend(undefined, undefined, false)
      );

      await assert.doesNotReject(
        ExtrinsicHelper.grantDelegation(
          delegatorKeys,
          providerKeys,
          signPayloadSr25519(delegatorKeys, addProviderData),
          payload
        ).signAndSend()
      );

      const delegation = await ExtrinsicHelper.apiPromise.query.msa.delegatorAndProviderToDelegation(
        newMsaId!,
        providerId
      );

      assert(delegation.isSome, 'delegation should exist');
      assert.equal(delegation.unwrap().revokedAt.toNumber(), 0, 'delegation revokedAt should be zero');
    });

    it('should revoke a delegation by delegator and retire msa', async function () {
      const delegatorKeys = createKeys();
      const payload = await generateDelegationPayload({ authorizedMsaId: providerId, schemaIds: [schemaId] });
      const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload);
      const op = ExtrinsicHelper.createSponsoredAccountWithDelegation(
        delegatorKeys,
        providerKeys,
        signPayloadSr25519(delegatorKeys, addProviderData),
        payload
      );
      const { target: msaEvent } = await op.fundAndSend(fundingSource, false);
      const newMsaId = msaEvent?.data.msaId;
      assert.notEqual(newMsaId, undefined, 'should have returned an MSA');
      await assert.doesNotReject(
        ExtrinsicHelper.revokeDelegationByProvider(newMsaId!, providerKeys).signAndSend(undefined, undefined, false)
      );

      const retireMsaOp = ExtrinsicHelper.retireMsa(delegatorKeys);
      const { target: retireMsaEvent } = await retireMsaOp.signAndSend();
      assert.notEqual(retireMsaEvent, undefined, 'should have returned MsaRetired event');
      assert.deepEqual(retireMsaEvent?.data.msaId, newMsaId, 'msaId should be equal');
    });

    it('should fail to retire msa with any active delegations', async function () {
      const delegatorKeys = createKeys();
      const payload = await generateDelegationPayload({ authorizedMsaId: providerId, schemaIds: [schemaId] });
      const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload);
      const op = ExtrinsicHelper.createSponsoredAccountWithDelegation(
        delegatorKeys,
        providerKeys,
        signPayloadSr25519(delegatorKeys, addProviderData),
        payload
      );
      const { target: msaEvent } = await op.fundAndSend(fundingSource, false);
      const newMsaId = msaEvent?.data.msaId;
      assert.notEqual(newMsaId, undefined, 'should have returned an MSA');
      const retireMsaOp = ExtrinsicHelper.retireMsa(delegatorKeys);
      await assert.rejects(retireMsaOp.signAndSend('current'), { name: 'RpcError', message: /Custom error: 6$/ });
    });
  });
});

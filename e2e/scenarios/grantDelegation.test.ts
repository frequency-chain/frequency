import '@frequency-chain/api-augment';
import { KeyringPair } from '@polkadot/keyring/types';
import { u16, u64 } from '@polkadot/types';
import assert from 'assert';
import { AddProviderPayload, Extrinsic, ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import {
  DOLLARS,
  createAndFundKeypair,
  createAndFundKeypairs,
  createKeys,
  generateDelegationPayload,
  getBlockNumber,
  signPayloadSr25519,
} from '../scaffolding/helpers';
import { SchemaGrantResponse, SchemaId } from '@frequency-chain/api-augment/interfaces';
import { getFundingSource } from '../scaffolding/funding';

const fundingSource = getFundingSource('scenarios-grant-delegation');

describe('Delegation Scenario Tests', function () {
  let keys: KeyringPair;
  let otherMsaKeys: KeyringPair;
  let noMsaKeys: KeyringPair;
  let providerKeys: KeyringPair;
  let otherProviderKeys: KeyringPair;
  let schemaId: u16;
  let schemaId2: SchemaId;
  let providerId: u64;
  let otherProviderId: u64;
  let msaId: u64;
  let otherMsaId: u64;

  before(async function () {
    // Fund all the different keys
    [noMsaKeys, keys, otherMsaKeys, providerKeys, otherProviderKeys] = await createAndFundKeypairs(
      fundingSource,
      ['noMsaKeys', 'keys', 'otherMsaKeys', 'providerKeys', 'otherProviderKeys'],
      1n * DOLLARS
    );

    const { target: msaCreatedEvent1 } = await ExtrinsicHelper.createMsa(keys).signAndSend();
    msaId = msaCreatedEvent1!.data.msaId;

    const { target: msaCreatedEvent2 } = await ExtrinsicHelper.createMsa(otherMsaKeys).signAndSend();
    otherMsaId = msaCreatedEvent2!.data.msaId;

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

    const createSchemaOp = ExtrinsicHelper.createSchema(
      keys,
      {
        type: 'record',
        name: 'Post',
        fields: [
          {
            name: 'title',
            type: {
              name: 'Title',
              type: 'string',
            },
          },
          {
            name: 'content',
            type: {
              name: 'Content',
              type: 'string',
            },
          },
          {
            name: 'fromId',
            type: {
              name: 'DSNPId',
              type: 'fixed',
              size: 8,
            },
          },
          {
            name: 'objectId',
            type: 'DSNPId',
          },
        ],
      },
      'AvroBinary',
      'OnChain'
    );
    let { target: createSchemaEvent } = await createSchemaOp.signAndSend();
    assert.notEqual(createSchemaEvent, undefined, 'setup should return SchemaCreated event');
    schemaId = createSchemaEvent!.data.schemaId;

    // Create a second schema
    ({ target: createSchemaEvent } = await createSchemaOp.signAndSend());
    assert.notEqual(createSchemaEvent, undefined, 'setup should return SchemaCreated event');
    schemaId2 = createSchemaEvent!.data.schemaId;
  });

  describe('delegation grants', function () {
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
  });

  describe('revoke schema permissions', function () {
    it('should fail to revoke schema permissions from non-MSA (NoKeyExists)', async function () {
      const nonMsaKeys = await createAndFundKeypair(fundingSource, 5_000_000n);
      const op = ExtrinsicHelper.revokeSchemaPermissions(nonMsaKeys, providerId, [schemaId]);
      await assert.rejects(op.fundAndSend(fundingSource), { name: 'NoKeyExists' });
    });

    it('should fail to revoke schema permissions from a provider for which no delegation exists (DelegationNotFound)', async function () {
      const op = ExtrinsicHelper.revokeSchemaPermissions(keys, otherProviderId, [schemaId]);
      await assert.rejects(op.fundAndSend(fundingSource), { name: 'DelegationNotFound' });
    });

    it('should fail to revoke schema permission which was never granted (SchemaNotGranted)', async function () {
      const schema = {
        name: 'nonGrantedSchema',
        type: 'record',
        fields: [],
      };

      const sop = ExtrinsicHelper.createSchema(keys, schema, 'AvroBinary', 'OnChain');
      const { target: sevent } = await sop.fundAndSend(fundingSource);
      const sid = sevent?.data.schemaId;
      assert.notEqual(sid, undefined, 'should have created schema');

      const op = ExtrinsicHelper.revokeSchemaPermissions(keys, providerId, [sid]);
      await assert.rejects(op.fundAndSend(fundingSource), { name: 'SchemaNotGranted' });
    });

    it('should successfully revoke granted schema', async function () {
      const op = ExtrinsicHelper.revokeSchemaPermissions(keys, providerId, [schemaId2]);
      await assert.doesNotReject(op.fundAndSend(fundingSource));
      const grants = await ExtrinsicHelper.apiPromise.rpc.msa.grantedSchemaIdsByMsaId(msaId, providerId);
      const grantedSchemaIds = grants
        .unwrap()
        .filter((grant) => grant.revoked_at.toBigInt() === 0n)
        .map((grant) => grant.schema_id.toNumber());
      assert.deepEqual(
        grantedSchemaIds,
        [schemaId.toNumber()],
        'granted schema permissions should include only non-revoked schema permission'
      );
    });
  });

  describe('revoke delegations', function () {
    it('should fail to revoke a delegation if no MSA exists (InvalidMsaKey)', async function () {
      const nonMsaKeys = await createAndFundKeypair(fundingSource);
      const op = ExtrinsicHelper.revokeDelegationByDelegator(nonMsaKeys, providerId);
      await assert.rejects(op.signAndSend()), { name: 'RpcError', message: /Custom error: 1$/ });
    });

    it('should revoke a delegation by delegator', async function () {
      const revokeDelegationOp = ExtrinsicHelper.revokeDelegationByDelegator(keys, providerId);
      const { target: revokeDelegationEvent } = await revokeDelegationOp.signAndSend();
      assert.notEqual(revokeDelegationEvent, undefined, 'should have returned DelegationRevoked event');
      assert.deepEqual(revokeDelegationEvent?.data.providerId, providerId, 'provider ids should be equal');
      assert.deepEqual(revokeDelegationEvent?.data.delegatorId, msaId, 'delegator ids should be equal');
    });

    it('should fail to revoke a delegation that has already been revoked (InvalidDelegation)', async function () {
      const op = ExtrinsicHelper.revokeDelegationByDelegator(keys, providerId);
      await assert.rejects(op.signAndSend(), { name: 'RpcError', message: /Custom error: 0$/ });
    });

    it('should fail to revoke delegation where no delegation exists (DelegationNotFound)', async function () {
      const op = ExtrinsicHelper.revokeDelegationByDelegator(keys, otherProviderId);
      await assert.rejects(op.signAndSend(), { name: 'RpcError', message: /Custom error: 0$/ });
    });

    describe('Successful revocation', function () {
      let newKeys: KeyringPair;
      let msaId: u64 | undefined;
      let revokedAtBlock: number;

      before(async function () {
        newKeys = createKeys();
        const payload = await generateDelegationPayload({ authorizedMsaId: providerId, schemaIds: [schemaId] });
        const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload);
        const op = ExtrinsicHelper.createSponsoredAccountWithDelegation(
          newKeys,
          providerKeys,
          signPayloadSr25519(newKeys, addProviderData),
          payload
        );
        const { target: msaEvent } = await op.fundAndSend(fundingSource);
        msaId = msaEvent?.data.msaId;
        assert.notEqual(msaId, undefined, 'should have returned an MSA');
      });

      it('schema permissions revoked block of delegation should be zero', async function () {
        const delegationsResponse = await ExtrinsicHelper.apiPromise.rpc.msa.grantedSchemaIdsByMsaId(msaId, providerId);
        assert(delegationsResponse.isSome);
        const delegations: SchemaGrantResponse[] = delegationsResponse.unwrap().toArray();
        delegations.forEach((delegation) => {
          assert(delegation.revoked_at.toBigInt() == 0n);
        });
      });

      it('should revoke a delegation by provider', async function () {
        const op = ExtrinsicHelper.revokeDelegationByProvider(msaId as u64, providerKeys);
        const { target: revokeEvent } = await op.signAndSend();
        assert.notEqual(revokeEvent, undefined, 'should have returned a DelegationRevoked event');
        assert.deepEqual(revokeEvent?.data.delegatorId, msaId, 'delegator ids should match');
        assert.deepEqual(revokeEvent?.data.providerId, providerId, 'provider ids should match');
        revokedAtBlock = await getBlockNumber();
      });

      it('revoked delegation should be reflected in all previously-granted schema permissions', async function () {
        // Make a block first to make sure the state has rolled to the next block
        const currentBlock = await getBlockNumber();
        ExtrinsicHelper.runToBlock(currentBlock + 1);
        const delegationsResponse = await ExtrinsicHelper.apiPromise.rpc.msa.grantedSchemaIdsByMsaId(msaId, providerId);
        assert(delegationsResponse.isSome);
        const delegations: SchemaGrantResponse[] = delegationsResponse.unwrap().toArray();
        delegations.forEach((delegation) => {
          const diff = delegation.revoked_at.toNumber() - revokedAtBlock;
          // Due to parallelization, this could be off by a few blocks
          assert(Math.abs(Number(diff.toString())) < 20);
        });
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
        const { target: msaEvent } = await op.fundAndSend(fundingSource);
        const newMsaId = msaEvent?.data.msaId;
        assert.notEqual(newMsaId, undefined, 'should have returned an MSA');
        await assert.doesNotReject(
          ExtrinsicHelper.revokeDelegationByProvider(newMsaId!, providerKeys).signAndSend()
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
        const { target: msaEvent } = await op.fundAndSend(fundingSource);
        const newMsaId = msaEvent?.data.msaId;
        assert.notEqual(newMsaId, undefined, 'should have returned an MSA');
        const retireMsaOp = ExtrinsicHelper.retireMsa(delegatorKeys);
        await assert.rejects(retireMsaOp.signAndSend(), { name: 'RpcError', message: /Custom error: 6$/ });
      });
    });
  });

  describe('createSponsoredAccountWithDelegation', function () {
    let sponsorKeys: KeyringPair;
    let op: Extrinsic;
    let defaultPayload: AddProviderPayload;

    before(async function () {
      sponsorKeys = await createAndFundKeypair(fundingSource, 50_000_000n);
      defaultPayload = {
        authorizedMsaId: providerId,
        schemaIds: [schemaId],
      };
    });

    it("should fail to create delegated account if provider ids don't match (UnauthorizedProvider)", async function () {
      const payload = await generateDelegationPayload({ ...defaultPayload, authorizedMsaId: otherProviderId });
      const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload);

      op = ExtrinsicHelper.createSponsoredAccountWithDelegation(
        sponsorKeys,
        providerKeys,
        signPayloadSr25519(sponsorKeys, addProviderData),
        payload
      );
      await assert.rejects(op.fundAndSend(fundingSource), { name: 'UnauthorizedProvider' });
    });

    it('should fail to create delegated account if payload signature cannot be verified (InvalidSignature)', async function () {
      const payload = await generateDelegationPayload({ ...defaultPayload, schemaIds: [] });
      const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload);

      op = ExtrinsicHelper.createSponsoredAccountWithDelegation(
        sponsorKeys,
        providerKeys,
        signPayloadSr25519(sponsorKeys, addProviderData),
        { ...payload, ...defaultPayload }
      );
      await assert.rejects(op.fundAndSend(fundingSource), { name: 'InvalidSignature' });
    });

    it('should fail to create delegated account if no MSA exists for origin (NoKeyExists)', async function () {
      const payload = await generateDelegationPayload(defaultPayload);
      const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload);

      op = ExtrinsicHelper.createSponsoredAccountWithDelegation(
        sponsorKeys,
        noMsaKeys,
        signPayloadSr25519(sponsorKeys, addProviderData),
        payload
      );
      await assert.rejects(op.fundAndSend(fundingSource), { name: 'NoKeyExists' });
    });

    it('should fail to create delegated account if MSA already exists for delegator (KeyAlreadyRegistered)', async function () {
      const payload = await generateDelegationPayload(defaultPayload);
      const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload);

      op = ExtrinsicHelper.createSponsoredAccountWithDelegation(
        keys,
        providerKeys,
        signPayloadSr25519(keys, addProviderData),
        payload
      );
      await assert.rejects(op.fundAndSend(fundingSource), { name: 'KeyAlreadyRegistered' });
    });

    it('should fail to create delegated account if provider is not registered (ProviderNotRegistered)', async function () {
      const payload = await generateDelegationPayload({ ...defaultPayload, authorizedMsaId: otherMsaId });
      const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload);

      op = ExtrinsicHelper.createSponsoredAccountWithDelegation(
        keys,
        otherMsaKeys,
        signPayloadSr25519(keys, addProviderData),
        payload
      );
      await assert.rejects(op.fundAndSend(fundingSource), { name: 'ProviderNotRegistered' });
    });

    it('should fail to create delegated account if provider if payload proof is too far in the future (ProofNotYetValid)', async function () {
      const payload = await generateDelegationPayload(defaultPayload, 999);
      const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload);

      op = ExtrinsicHelper.createSponsoredAccountWithDelegation(
        sponsorKeys,
        providerKeys,
        signPayloadSr25519(sponsorKeys, addProviderData),
        payload
      );
      await assert.rejects(op.fundAndSend(fundingSource), { name: 'ProofNotYetValid' });
    });

    it('should fail to create delegated account if provider if payload proof has expired (ProofHasExpired))', async function () {
      const payload = await generateDelegationPayload(defaultPayload, -1);
      const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload);

      op = ExtrinsicHelper.createSponsoredAccountWithDelegation(
        sponsorKeys,
        providerKeys,
        signPayloadSr25519(sponsorKeys, addProviderData),
        payload
      );
      await assert.rejects(op.fundAndSend(fundingSource), { name: 'ProofHasExpired' });
    });

    it('should successfully create a delegated account', async function () {
      const payload = await generateDelegationPayload(defaultPayload);
      const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload);

      op = ExtrinsicHelper.createSponsoredAccountWithDelegation(
        sponsorKeys,
        providerKeys,
        signPayloadSr25519(sponsorKeys, addProviderData),
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

import '@frequency-chain/api-augment';
import type { KeyringPair } from '@polkadot/keyring/types';
import { u16, u64 } from '@polkadot/types';
import assert from 'assert';
import { Extrinsic, ExtrinsicHelper, type AddProviderPayload } from '../scaffolding/extrinsicHelpers';
import {
  DOLLARS,
  createAndFundKeypairs,
  createMsaAndProvider,
  generateDelegationPayload,
  signPayloadSr25519,
  getOrCreateIntentAndSchema, getOrCreateDelegationSchema,
} from '../scaffolding/helpers';
import { getFundingSource } from '../scaffolding/funding';

let fundingSource: KeyringPair;

describe('Delegation Scenario Tests createSponsoredAccountWithDelegation', function () {
  let keys: KeyringPair;
  let sponsorKeys: KeyringPair;
  let otherMsaKeys: KeyringPair;
  let noMsaKeys: KeyringPair;
  let providerKeys: KeyringPair;
  let otherProviderKeys: KeyringPair;
  let intentId: u16;
  let providerId: u64;
  let otherProviderId: u64;
  let otherMsaId: u64;
  let op: Extrinsic;
  let defaultPayload: AddProviderPayload;

  before(async function () {
    fundingSource = await getFundingSource(import.meta.url);
    // Fund all the different keys
    [noMsaKeys, sponsorKeys, keys, otherMsaKeys, providerKeys, otherProviderKeys] = await createAndFundKeypairs(
      fundingSource,
      ['noMsaKeys', 'sponsorKeys', 'keys', 'otherMsaKeys', 'providerKeys', 'otherProviderKeys'],
      1n * DOLLARS
    );

    let _msaCreatedEvent1: any, msaCreatedEvent2: any;
    // eslint-disable-next-line prefer-const
    [{ target: _msaCreatedEvent1 }, { target: msaCreatedEvent2 }, { intentId }] = await Promise.all([
      ExtrinsicHelper.createMsa(keys).signAndSend(),
      ExtrinsicHelper.createMsa(otherMsaKeys).signAndSend(),
      getOrCreateDelegationSchema(fundingSource),
    ]);

    otherMsaId = msaCreatedEvent2!.data.msaId;

    [providerId, otherProviderId] = await Promise.all([
      createMsaAndProvider(fundingSource, providerKeys, 'MyPoster'),
      createMsaAndProvider(fundingSource, otherProviderKeys, 'MyPoster2'),
    ]);
    assert.notEqual(providerId, undefined, 'setup should return a Provider Id for Provider 1');
    assert.notEqual(otherProviderId, undefined, 'setup should return a Provider Id for Provider 2');

    defaultPayload = {
      authorizedMsaId: providerId,
      intentIds: [intentId],
    };
    // Make sure we are finalized before all the tests
    await ExtrinsicHelper.waitForFinalization();
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
    const payload = await generateDelegationPayload({ ...defaultPayload, intentIds: [] });
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

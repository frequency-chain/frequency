import '@frequency-chain/api-augment';
import { KeyringPair } from '@polkadot/keyring/types';
import { Bytes, u64, u16 } from '@polkadot/types';
import assert from 'assert';
import { AddKeyData, ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import {
  createKeys,
  createAndFundKeypair,
  createMsaAndProvider,
  generateDelegationPayload,
  getBlockNumber,
  signPayloadSr25519,
  stakeToProvider,
  generateAddKeyPayload,
  CENTS,
  DOLLARS,
  getOrCreateGraphChangeSchema,
  getTestHandle,
  assertAddNewKey,
} from '../scaffolding/helpers';
import { getFundingSource } from '../scaffolding/funding';
import { getUnifiedPublicKey } from '@frequency-chain/ethereum-utils';

const FUNDS_AMOUNT: bigint = 50n * DOLLARS;
const fundingSource = getFundingSource(import.meta.url);

describe('Capacity Transaction Failures', function () {
  describe('pay_with_capacity', function () {
    describe('when caller does not have a Capacity account', function () {
      let delegatorKeys: KeyringPair;
      let delegatorProviderId: u64;
      let schemaId: u16;

      beforeEach(async function () {
        // Create and fund a keypair with EXISTENTIAL_DEPOSIT
        // Use this keypair for delegator operations
        delegatorKeys = createKeys('OtherProviderKeys');
        delegatorProviderId = await createMsaAndProvider(fundingSource, delegatorKeys, 'Delegator', FUNDS_AMOUNT);
        schemaId = new u16(ExtrinsicHelper.api.registry, 0);
      });

      describe('but has an MSA account that has not been registered as a Provider', function () {
        it('fails to pay for a transaction', async function () {
          // Create a keypair with msaId, but no provider
          const noProviderKeys = await createAndFundKeypair(fundingSource, FUNDS_AMOUNT, 'NoProviderKeys');

          const createMsaOp = ExtrinsicHelper.createMsa(noProviderKeys);

          const { target: msaCreatedEvent } = await createMsaOp.fundAndSend(fundingSource);
          assert.notEqual(msaCreatedEvent, undefined, 'should have returned MsaCreated event');

          // When a user is not a registered provider and attempts to pay with Capacity,
          // it should error with InvalidTransaction::Payment, which is a 1010 error, Inability to pay some fees.
          const payload = await generateDelegationPayload({
            authorizedMsaId: delegatorProviderId,
            schemaIds: [schemaId],
          });
          const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload);
          const grantDelegationOp = ExtrinsicHelper.grantDelegation(
            delegatorKeys,
            noProviderKeys,
            signPayloadSr25519(delegatorKeys, addProviderData),
            payload
          );

          await assert.rejects(grantDelegationOp.payWithCapacity('current'), {
            name: 'RpcError',
            message: '1010: Invalid Transaction: Inability to pay some fees , e.g. account balance too low',
          });
        });
      });

      describe('and does not have an MSA account associated to signing keys', function () {
        it('fails to pay for a transaction', async function () {
          const emptyKeys = await createAndFundKeypair(fundingSource, 50_000_000n);

          const payload = await generateDelegationPayload({
            authorizedMsaId: delegatorProviderId,
            schemaIds: [schemaId],
          });
          const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload);
          const grantDelegationOp = ExtrinsicHelper.grantDelegation(
            delegatorKeys,
            emptyKeys,
            signPayloadSr25519(delegatorKeys, addProviderData),
            payload
          );

          await assert.rejects(grantDelegationOp.payWithCapacity('current'), {
            name: 'RpcError',
            message: '1010: Invalid Transaction: Custom error: 1',
          });
        });
      });
    });

    describe('when caller has a Capacity account', function () {
      let schemaId: u16;
      const amountStaked = 3n * DOLLARS;

      before(async function () {
        // Create schemas for testing with Grant Delegation to test pay_with_capacity
        schemaId = await getOrCreateGraphChangeSchema(fundingSource);
        assert.notEqual(schemaId, undefined, 'setup should populate schemaId');
      });

      it('fails when a provider makes an eligible extrinsic call using non-funded control key', async function () {
        const capacityKeys = createKeys('CapacityKeysNSF');
        const capacityProvider = await createMsaAndProvider(fundingSource, capacityKeys, 'CapProvNSF', FUNDS_AMOUNT);
        // this will first fund 'capacityKeys' before staking
        await assert.doesNotReject(stakeToProvider(fundingSource, fundingSource, capacityProvider, amountStaked));

        // As current owner, add a new set of control keys that do not have a balance.
        const newControlKeypair = createKeys('NewKeyNoBalance');
        const newPublicKey = getUnifiedPublicKey(newControlKeypair);
        const addKeyPayload: AddKeyData = await generateAddKeyPayload({
          msaId: capacityProvider,
          newPublicKey: newPublicKey,
        });
        await assertAddNewKey(capacityKeys, addKeyPayload, newControlKeypair);

        // attempt a capacity transaction using the new unfunded key: claimHandle
        const handle = getTestHandle();
        const expiration = (await getBlockNumber()) + 10;
        const handle_vec = new Bytes(ExtrinsicHelper.api.registry, handle);
        const handlePayload = {
          baseHandle: handle_vec,
          expiration: expiration,
        };
        const claimHandlePayload = ExtrinsicHelper.api.registry.createType(
          'CommonPrimitivesHandlesClaimHandlePayload',
          handlePayload
        );
        const claimHandle = ExtrinsicHelper.claimHandle(newControlKeypair, claimHandlePayload);
        await assert.rejects(claimHandle.payWithCapacity('current'), {
          name: 'RpcError',
          message: '1010: Invalid Transaction: Inability to pay some fees , e.g. account balance too low',
        });
      });

      // When a user attempts to pay for a non-capacity transaction with Capacity,
      // it should error and drop the transaction from the transaction-pool.
      it('fails to pay with Capacity for a non-capacity transaction', async function () {
        const capacityKeys = createKeys('CapacityKeys');
        const capacityProvider = await createMsaAndProvider(
          fundingSource,
          capacityKeys,
          'CapacityProvider',
          FUNDS_AMOUNT
        );
        const nonCapacityTxn = ExtrinsicHelper.stake(capacityKeys, capacityProvider, 1n * CENTS);
        await assert.rejects(nonCapacityTxn.payWithCapacity('current'), {
          name: 'RpcError',
          message: '1010: Invalid Transaction: Custom error: 0',
        });
      });

      // When a user does not have enough capacity to pay for the transaction fee
      // and is NOT eligible to replenish Capacity, it should error and be dropped
      // from the transaction pool.
      it('fails to pay for a transaction with empty capacity', async function () {
        const noCapacityKeys = createKeys('noCapacityKeys');
        const noCapacityProvider = await createMsaAndProvider(
          fundingSource,
          noCapacityKeys,
          'NoCapProvider',
          undefined,
          false
        );

        const delegatorKeys = createKeys('delegatorKeys');

        const payload = await generateDelegationPayload({
          authorizedMsaId: noCapacityProvider,
          schemaIds: [schemaId],
        });
        const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload);
        const grantDelegationOp = ExtrinsicHelper.createSponsoredAccountWithDelegation(
          delegatorKeys,
          noCapacityKeys,
          signPayloadSr25519(noCapacityKeys, addProviderData),
          payload
        );

        await assert.rejects(grantDelegationOp.payWithCapacity('current'), {
          name: 'RpcError',
          message: '1010: Invalid Transaction: Inability to pay some fees , e.g. account balance too low',
        });
      });

      // *All keys should have at least an EXISTENTIAL_DEPOSIT = 1M.
      it('fails to pay for transaction when key does has not met the min deposit', async function () {
        const capacityKeys = createKeys('CapKeysUnder');
        const capacityProvider = await createMsaAndProvider(fundingSource, capacityKeys, 'CapProvUnder', FUNDS_AMOUNT);
        const noTokensKeys = createKeys('noTokensKeys');
        const delegatorKeys = createKeys('delegatorKeys');

        await assert.doesNotReject(stakeToProvider(fundingSource, fundingSource, capacityProvider, 1n * DOLLARS));

        // Add new key
        const newKeyPayload: AddKeyData = await generateAddKeyPayload({
          msaId: new u64(ExtrinsicHelper.api.registry, capacityProvider),
          newPublicKey: getUnifiedPublicKey(noTokensKeys),
        });
        const addKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', newKeyPayload);

        const ownerSig = signPayloadSr25519(capacityKeys, addKeyData);
        const newSig = signPayloadSr25519(noTokensKeys, addKeyData);
        const addPublicKeyOp = ExtrinsicHelper.addPublicKeyToMsa(capacityKeys, ownerSig, newSig, newKeyPayload);

        const { target: publicKeyEvent } = await addPublicKeyOp.fundAndSend(fundingSource);
        assert.notEqual(publicKeyEvent, undefined, 'should have added public key');

        const payload = await generateDelegationPayload({
          authorizedMsaId: capacityProvider,
          schemaIds: [schemaId],
        });
        const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', payload);
        const grantDelegationOp = ExtrinsicHelper.createSponsoredAccountWithDelegation(
          delegatorKeys,
          noTokensKeys,
          signPayloadSr25519(delegatorKeys, addProviderData),
          payload
        );

        await assert.rejects(grantDelegationOp.payWithCapacity('current'), {
          name: 'RpcError',
          message: '1010: Invalid Transaction: Inability to pay some fees , e.g. account balance too low',
        });
      });
    });
  });
});

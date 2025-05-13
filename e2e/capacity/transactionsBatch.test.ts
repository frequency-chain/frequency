import '@frequency-chain/api-augment';
import { KeyringPair } from '@polkadot/keyring/types';
import { Bytes, u64, u16 } from '@polkadot/types';
import { u8aToHex } from '@polkadot/util/u8a/toHex';
import { u8aWrapBytes } from '@polkadot/util';
import assert from 'assert';
import { AddProviderPayload, ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import {
  createKeys,
  createMsaAndProvider,
  generateDelegationPayload,
  getBlockNumber,
  signPayloadSr25519,
  stakeToProvider,
  DOLLARS,
  getTestHandle,
} from '../scaffolding/helpers';
import { getFundingSource } from '../scaffolding/funding';
import { getUnifiedPublicKey } from '@frequency-chain/ethereum-utils/address';

const FUNDS_AMOUNT: bigint = 50n * DOLLARS;
const fundingSource = getFundingSource(import.meta.url);

describe('Capacity Transactions Batch', function () {
  describe('pay_with_capacity_batch_all', function () {
    let capacityProviderKeys: KeyringPair;
    let capacityProvider: u64;
    let defaultPayload: AddProviderPayload;
    const amountStaked = 9n * DOLLARS;

    beforeEach(async function () {
      const schemaId: u16 = new u16(ExtrinsicHelper.api.registry, 1);
      capacityProviderKeys = createKeys('CapacityProviderKeys');
      capacityProvider = await createMsaAndProvider(
        fundingSource,
        capacityProviderKeys,
        'CapacityProvider',
        FUNDS_AMOUNT
      );
      defaultPayload = {
        authorizedMsaId: capacityProvider,
        schemaIds: [schemaId],
      };
    });

    it('successfully pays with Capacity for a batch of eligible transactions - [createSponsoredAccountWithDelegation, claimHandle]', async function () {
      await assert.doesNotReject(stakeToProvider(fundingSource, fundingSource, capacityProvider, amountStaked));

      const addProviderPayload = await generateDelegationPayload({ ...defaultPayload });
      const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', addProviderPayload);
      const delegatorKeys = createKeys('delegatorKeys');
      const createSponsoredAccountWithDelegation = ExtrinsicHelper.api.tx.msa.createSponsoredAccountWithDelegation(
        getUnifiedPublicKey(delegatorKeys),
        signPayloadSr25519(delegatorKeys, addProviderData),
        addProviderPayload
      );

      const handle = getTestHandle();
      const handle_vec = new Bytes(ExtrinsicHelper.api.registry, handle);
      const expiration = (await getBlockNumber()) + 5;
      const handlePayload = {
        baseHandle: handle_vec,
        expiration: expiration,
      };
      const claimHandlePayload: any = ExtrinsicHelper.api.registry.createType(
        'CommonPrimitivesHandlesClaimHandlePayload',
        handlePayload
      );
      const claimHandleProof = {
        Sr25519: u8aToHex(delegatorKeys.sign(u8aWrapBytes(claimHandlePayload.toU8a()))),
      };

      const claimHandle = ExtrinsicHelper.api.tx.handles.claimHandle(
        getUnifiedPublicKey(delegatorKeys),
        claimHandleProof,
        claimHandlePayload
      );
      const calls = [createSponsoredAccountWithDelegation, claimHandle];

      const payWithCapacityBatchAllOp = ExtrinsicHelper.payWithCapacityBatchAll(capacityProviderKeys, calls);

      const { target: batchCompletedEvent, eventMap } = await payWithCapacityBatchAllOp.signAndSend();

      if (batchCompletedEvent && !ExtrinsicHelper.api.events.utility.BatchCompleted.is(batchCompletedEvent)) {
        assert.fail('should return a BatchCompletedEvent');
      }

      assert.notEqual(eventMap['msa.DelegationGranted'], undefined, 'should have returned DelegationGranted event');
      assert.notEqual(eventMap['handles.HandleClaimed'], undefined, 'should have returned HandleClaimed event');
    });

    it('batch fails if one transaction fails - [createSponsoredAccountWithDelegation, claimHandle]', async function () {
      await assert.doesNotReject(stakeToProvider(fundingSource, fundingSource, capacityProvider, amountStaked));

      const addProviderPayload = await generateDelegationPayload({ ...defaultPayload });
      const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', addProviderPayload);
      const delegatorKeys = createKeys('delegatorKeys');
      const createSponsoredAccountWithDelegation = ExtrinsicHelper.api.tx.msa.createSponsoredAccountWithDelegation(
        getUnifiedPublicKey(delegatorKeys),
        signPayloadSr25519(delegatorKeys, addProviderData),
        addProviderPayload
      );

      const handle = 'test_handle_that_exceeds_the_byte_limit';
      const handle_vec = new Bytes(ExtrinsicHelper.api.registry, handle);
      const expiration = (await getBlockNumber()) + 5;
      const handlePayload = {
        baseHandle: handle_vec,
        expiration: expiration,
      };
      const claimHandlePayload: any = ExtrinsicHelper.api.registry.createType(
        'CommonPrimitivesHandlesClaimHandlePayload',
        handlePayload
      );
      const calimHandleProof = {
        Sr25519: u8aToHex(delegatorKeys.sign(u8aWrapBytes(claimHandlePayload.toU8a()))),
      };

      const claimHandle = ExtrinsicHelper.api.tx.handles.claimHandle(
        getUnifiedPublicKey(delegatorKeys),
        calimHandleProof,
        claimHandlePayload
      );
      const calls = [createSponsoredAccountWithDelegation, claimHandle];

      const payWithCapacityBatchAllOp = ExtrinsicHelper.payWithCapacityBatchAll(capacityProviderKeys, calls);

      await assert.rejects(payWithCapacityBatchAllOp.signAndSend(), {
        name: 'InvalidHandleByteLength',
      });
    });
  });
});

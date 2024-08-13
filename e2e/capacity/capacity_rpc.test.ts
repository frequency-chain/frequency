import '@frequency-chain/api-augment';
import { KeyringPair } from '@polkadot/keyring/types';
import { u64, u16 } from '@polkadot/types';
import assert from 'assert';
import { AddProviderPayload, ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import {
  createKeys,
  createMsaAndProvider,
  generateDelegationPayload,
  signPayloadSr25519,
  DOLLARS,
  getOrCreateGraphChangeSchema,
} from '../scaffolding/helpers';
import { FeeDetails } from '@polkadot/types/interfaces';
import { getFundingSource } from '../scaffolding/funding';

const FUNDS_AMOUNT: bigint = 50n * DOLLARS;
const fundingSource = getFundingSource('capacity-rpcs');

describe('Capacity RPC', function () {
  let capacityProviderKeys: KeyringPair;
  let capacityProvider: u64;
  let schemaId: u16;
  let defaultPayload: AddProviderPayload;

  before(async function () {
    // Create schemas for testing with Grant Delegation to test pay_with_capacity
    schemaId = await getOrCreateGraphChangeSchema(fundingSource);
    assert.notEqual(schemaId, undefined, 'setup should populate schemaId');
  });

  beforeEach(async function () {
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

  it('Returns `FeeDetails` when requesting capacity cost of a transaction', async function () {
    const addProviderPayload = await generateDelegationPayload({ ...defaultPayload });
    const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', addProviderPayload);
    const delegatorKeys = createKeys('delegatorKeys');
    const call = ExtrinsicHelper.api.tx.msa.createSponsoredAccountWithDelegation(
      delegatorKeys.publicKey,
      signPayloadSr25519(delegatorKeys, addProviderData),
      addProviderPayload
    );

    // Actual weights and fee
    const {
      weight: { refTime, proofSize },
    } = await ExtrinsicHelper.apiPromise.call.transactionPaymentApi.queryInfo(call.toHex(), 0);
    const weightFee = await ExtrinsicHelper.apiPromise.call.transactionPaymentApi.queryWeightToFee({
      refTime,
      proofSize,
    });

    const feeDetails: FeeDetails = await ExtrinsicHelper.apiPromise.rpc.frequencyTxPayment.computeCapacityFeeDetails(
      call.toHex(),
      null
    );
    assert.notEqual(feeDetails, undefined, 'should have returned a feeDetails');
    assert.notEqual(feeDetails.inclusionFee, undefined, 'should have returned a partialFee');
    assert(feeDetails.inclusionFee.isSome, 'should have returned a partialFee');
    const { baseFee, lenFee, adjustedWeightFee } = feeDetails.inclusionFee.toJSON() as any;
    const baseFeeSnapshot = 124103;
    assert(
      Math.abs(baseFee - baseFeeSnapshot) < 50_000,
      'The base fee appears to be wrong or have changed more than expected'
    );
    assert(Math.abs(lenFee - 1170000) < 100, 'The len fee appears to be wrong or have changed more than expected');
    // This is comparing stable weight, which has no impact from targeted_fee_adjustment, with actual weights.
    assert(
      Math.abs(adjustedWeightFee - weightFee.toNumber()) < 50_000,
      'The adjusted weight fee appears to be wrong or have changed more than expected'
    );
  });

  it('Returns `FeeDetails` when requesting capacity cost of a transaction when wrapped in payWithCapacity', async function () {
    const addProviderPayload = await generateDelegationPayload({ ...defaultPayload });
    const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', addProviderPayload);
    const delegatorKeys = createKeys('delegatorKeys');
    const insideTx = ExtrinsicHelper.api.tx.msa.createSponsoredAccountWithDelegation(
      delegatorKeys.publicKey,
      signPayloadSr25519(delegatorKeys, addProviderData),
      addProviderPayload
    );
    const tx = ExtrinsicHelper.api.tx.frequencyTxPayment.payWithCapacity(insideTx);
    const feeDetails: FeeDetails = await ExtrinsicHelper.apiPromise.rpc.frequencyTxPayment.computeCapacityFeeDetails(
      tx.toHex(),
      null
    );
    assert.notEqual(feeDetails, undefined, 'should have returned a feeDetails');
    assert.notEqual(feeDetails.inclusionFee, undefined, 'should have returned a partialFee');
    assert(feeDetails.inclusionFee.isSome, 'should have returned a partialFee');
    const { baseFee, lenFee, adjustedWeightFee } = feeDetails.inclusionFee.toJSON() as any;

    // payWithCapacity costs will fluctuate, so just checking that they are valid positive numbers
    assert(baseFee > 1, 'The base fee appears to be wrong.');
    assert(lenFee > 1, 'The len fee appears to be wrong.');
    assert(adjustedWeightFee > 1, 'The adjusted weight fee appears to be wrong');
  });

  it('Returns nothing when requesting capacity cost of a non-capacity transaction', async function () {
    const tx = ExtrinsicHelper.api.tx.msa.retireMsa();
    const feeDetails: FeeDetails = await ExtrinsicHelper.apiPromise.rpc.frequencyTxPayment.computeCapacityFeeDetails(
      tx.toHex(),
      null
    );
    assert.notEqual(feeDetails, undefined, 'should have returned a feeDetails');
    assert(feeDetails.inclusionFee.isNone, 'should have returned None for the inclusionFee');
  });

  it('Returns nothing when requesting pay with capacity call with a non-capacity transaction', async function () {
    const insideTx = ExtrinsicHelper.api.tx.msa.retireMsa();
    const tx = ExtrinsicHelper.api.tx.frequencyTxPayment.payWithCapacity(insideTx);
    const feeDetails: FeeDetails = await ExtrinsicHelper.apiPromise.rpc.frequencyTxPayment.computeCapacityFeeDetails(
      tx.toHex(),
      null
    );
    assert.notEqual(feeDetails, undefined, 'should have returned a feeDetails');
    assert(feeDetails.inclusionFee.isNone, 'should have returned None for the inclusionFee');
  });
});

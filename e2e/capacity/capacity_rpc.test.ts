import '@frequency-chain/api-augment';
import type { KeyringPair } from '@polkadot/keyring/types';
import { u64, u16 } from '@polkadot/types';
import assert from 'assert';
import { Extrinsic, ExtrinsicHelper, type AddProviderPayload } from '../scaffolding/extrinsicHelpers';
import {
  createKeys,
  createMsaAndProvider,
  generateDelegationPayload,
  signPayloadSr25519,
  DOLLARS,
  getOrCreateGraphChangeSchema,
  stakeToProvider,
} from '../scaffolding/helpers';
import { getFundingSource } from '../scaffolding/funding';
import { getUnifiedPublicKey } from '@frequency-chain/ethereum-utils';
import { firstValueFrom, tap } from 'rxjs';

const FUNDS_AMOUNT: bigint = 200n * DOLLARS;
const STAKED_AMOUNT: bigint = 50n * DOLLARS;
let fundingSource: KeyringPair;

describe('Capacity RPC', function () {
  let capacityProviderKeys: KeyringPair;
  let capacityProvider: u64;
  let schemaId: u16;
  let intentId: u16;
  let defaultPayload: AddProviderPayload;

  // Because this is testing non-executing code, we can do this beforeAll instead of beforeEach
  before(async function () {
    fundingSource = await getFundingSource(import.meta.url);
    // Create schemas for testing with Grant Delegation to test pay_with_capacity
    ({ intentId, schemaId } = await getOrCreateGraphChangeSchema(fundingSource));
    assert.notEqual(schemaId, undefined, 'setup should populate schemaId');
    capacityProviderKeys = createKeys('CapacityProviderKeys');
    capacityProvider = await createMsaAndProvider(
      fundingSource,
      capacityProviderKeys,
      'CapacityProvider',
      FUNDS_AMOUNT
    );
    await stakeToProvider(fundingSource, capacityProviderKeys, capacityProvider, STAKED_AMOUNT);
    defaultPayload = {
      authorizedMsaId: capacityProvider,
      intentIds: [schemaId],
    };
  });

  it('Returns `FeeDetails` when requesting capacity cost of a transaction', async function () {
    const addProviderPayload = await generateDelegationPayload({ ...defaultPayload });
    const addProviderData = ExtrinsicHelper.api.registry.createType('PalletMsaAddProvider', addProviderPayload);
    const delegatorKeys = createKeys('delegatorKeys');
    const call = ExtrinsicHelper.api.tx.msa.createSponsoredAccountWithDelegation(
      getUnifiedPublicKey(delegatorKeys),
      signPayloadSr25519(delegatorKeys, addProviderData),
      addProviderPayload
    );

    // Actual weights and fee
    const {
      weight: { refTime, proofSize },
    } = await ExtrinsicHelper.apiPromise.call.transactionPaymentApi.queryInfo(call.toU8a(), call.length);
    // Why does it need to be call.toU8a() above instead of just call or call.toHex()?
    // https://github.com/polkadot-js/apps/issues/10994

    const weightFee = await ExtrinsicHelper.apiPromise.call.transactionPaymentApi.queryWeightToFee({
      refTime,
      proofSize,
    });

    const feeDetails = await ExtrinsicHelper.apiPromise.rpc.frequencyTxPayment.computeCapacityFeeDetails(
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
      getUnifiedPublicKey(delegatorKeys),
      signPayloadSr25519(delegatorKeys, addProviderData),
      addProviderPayload
    );
    const tx = ExtrinsicHelper.api.tx.frequencyTxPayment.payWithCapacity(insideTx);
    const signedTx = await firstValueFrom(tx.signAsync(capacityProviderKeys));
    const signedHex = signedTx.toHex();
    // it's important to submit a signed extrinsic to the rpc to get an accurate estimate due to the actual length of extrinsic and etc.
    const feeDetails = await ExtrinsicHelper.apiPromise.rpc.frequencyTxPayment.computeCapacityFeeDetails(
      signedHex,
      null
    );
    assert.notEqual(feeDetails, undefined, 'should have returned a feeDetails');
    assert.notEqual(feeDetails.inclusionFee, undefined, 'should have returned a partialFee');
    assert(feeDetails.inclusionFee.isSome, 'should have returned a partialFee');
    const { baseFee, lenFee, adjustedWeightFee } = feeDetails.inclusionFee.toJSON() as any;
    const estimatedCapacity = BigInt(baseFee + lenFee + adjustedWeightFee);

    const extrinsic = new Extrinsic(() => insideTx, capacityProviderKeys, ExtrinsicHelper.api.events.msa.MsaCreated);
    const { eventMap } = await extrinsic.payWithCapacity();
    const callCapacityCost = (eventMap['capacity.CapacityWithdrawn'].data as any).amount.toBigInt();

    // payWithCapacity costs will fluctuate, so just checking that they are valid positive numbers
    assert(baseFee > 1, 'The base fee appears to be wrong.');
    assert(lenFee > 1, 'The len fee appears to be wrong.');
    assert(adjustedWeightFee > 1, 'The adjusted weight fee appears to be wrong');
    assert(
      estimatedCapacity - callCapacityCost < 20_000,
      'The estimated capacity and the actual capacity are not within threshold'
    );
  });

  it('Returns nothing when requesting capacity cost of a non-capacity transaction', async function () {
    const tx = ExtrinsicHelper.api.tx.msa.retireMsa();
    const feeDetails = await ExtrinsicHelper.apiPromise.rpc.frequencyTxPayment.computeCapacityFeeDetails(
      tx.toHex(),
      null
    );
    assert.notEqual(feeDetails, undefined, 'should have returned a feeDetails');
    assert(feeDetails.inclusionFee.isNone, 'should have returned None for the inclusionFee');
  });

  it('Returns nothing when requesting pay with capacity call with a non-capacity transaction', async function () {
    const insideTx = ExtrinsicHelper.api.tx.msa.retireMsa();
    const tx = ExtrinsicHelper.api.tx.frequencyTxPayment.payWithCapacity(insideTx);
    const feeDetails = await ExtrinsicHelper.apiPromise.rpc.frequencyTxPayment.computeCapacityFeeDetails(
      tx.toHex(),
      null
    );
    assert.notEqual(feeDetails, undefined, 'should have returned a feeDetails');
    assert(feeDetails.inclusionFee.isNone, 'should have returned None for the inclusionFee');
  });
});

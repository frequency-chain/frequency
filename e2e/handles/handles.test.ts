//  Handles Basic test suite
import '@frequency-chain/api-augment';
import assert from 'assert';
import { CENTS, createMsa, DOLLARS, getTestHandle } from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { MessageSourceId } from '@frequency-chain/api-augment/interfaces';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { Bytes } from '@polkadot/types';
import { getBlockNumber } from '../scaffolding/helpers';
import { hasRelayChain } from '../scaffolding/env';
import { getFundingSource } from '../scaffolding/funding';

const fundingSource = getFundingSource(import.meta.url);
const expirationOffset = hasRelayChain() ? 4 : 100;

describe('Handles: Claim and Retire', function () {
  let msaId: MessageSourceId;
  let msaOwnerKeys: KeyringPair;

  before(async function () {
    // Create a MSA for the delegator
    [msaId, msaOwnerKeys] = await createMsa(fundingSource, 50n * CENTS);
    assert.notEqual(msaOwnerKeys, undefined, 'setup should populate delegator_key');
    assert.notEqual(msaId, undefined, 'setup should populate msaId');
  });

  it('should be able to claim a handle', async function () {
    const handle = getTestHandle('A-test');
    const currentBlock = await getBlockNumber();
    const handle_vec = new Bytes(ExtrinsicHelper.api.registry, handle);
    const payload = {
      baseHandle: handle_vec,
      expiration: currentBlock + expirationOffset,
    };
    const claimHandlePayload = ExtrinsicHelper.api.registry.createType(
      'CommonPrimitivesHandlesClaimHandlePayload',
      payload
    );
    const claimHandle = ExtrinsicHelper.claimHandle(msaOwnerKeys, claimHandlePayload);
    const { target: event } = await claimHandle.fundAndSend(fundingSource);
    assert.notEqual(event, undefined, 'claimHandle should return an event');
    assert.notEqual(event!.data.handle.toString(), '', 'claimHandle should emit a handle');
  });

  it('should be able to retire a handle', async function () {
    const handle_response = await ExtrinsicHelper.getHandleForMSA(msaId);
    if (!handle_response.isSome) {
      throw new Error('handle_response should be Some');
    }
    const full_handle_state = handle_response.unwrap();
    const suffix_from_state = full_handle_state.suffix;
    const suffix = suffix_from_state.toNumber();
    assert.notEqual(suffix, 0, 'suffix should not be 0');
    assert.notEqual(full_handle_state.canonical_base, undefined, 'canonical_base should not be undefined');
    assert.notEqual(full_handle_state.base_handle, undefined, 'base_handle should not be undefined');
    const currentBlock = await getBlockNumber();
    // Must be at least 6 > original expiration to make sure we get past the finalization
    await ExtrinsicHelper.runToBlock(currentBlock + expirationOffset + 6);

    const retireHandle = ExtrinsicHelper.retireHandle(msaOwnerKeys);
    const { target: event } = await retireHandle.signAndSend();
    assert.notEqual(event, undefined, 'retireHandle should return an event');
    const handle = event!.data.handle.toString();
    assert.notEqual(handle, '', 'retireHandle should return the correct handle');
  });
});

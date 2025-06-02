//  Handles Alt test suite
import '@frequency-chain/api-augment';
import assert from 'assert';
import { CENTS, createMsa, getTestHandle } from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { MessageSourceId } from '@frequency-chain/api-augment/interfaces';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { Bytes } from '@polkadot/types';
import { getBlockNumber } from '../scaffolding/helpers';
import { hasRelayChain } from '../scaffolding/env';
import { getFundingSource } from '../scaffolding/funding';

const fundingSource = getFundingSource(import.meta.url);
const expirationOffset = hasRelayChain() ? 4 : 100;

describe('Handles: Claim and Retire Alt', function () {
  let msaId: MessageSourceId;
  let msaOwnerKeys: KeyringPair;

  before(async function () {
    // Create a MSA for the delegator
    [msaId, msaOwnerKeys] = await createMsa(fundingSource, 50n * CENTS);
    assert.notEqual(msaOwnerKeys, undefined, 'setup should populate delegator_key');
    assert.notEqual(msaId, undefined, 'setup should populate msaId');
  });

  describe('Claim Handle with possible presumptive suffix/RPC test', function () {
    it('should be able to claim a handle and check suffix (=suffix_assumed if available on chain)', async function () {
      const handle = getTestHandle('B-test');
      const handle_bytes = new Bytes(ExtrinsicHelper.api.registry, handle);
      /// Get presumptive suffix from chain (rpc)
      const suffixes_response = await ExtrinsicHelper.getNextSuffixesForHandle(handle, 10);
      const resp_base_handle = suffixes_response.base_handle.toString();
      assert.equal(resp_base_handle, handle, 'resp_base_handle should be equal to handle');
      const suffix_assumed = suffixes_response.suffixes[0];
      assert.notEqual(suffix_assumed, 0, 'suffix_assumed should not be 0');

      const currentBlock = await getBlockNumber();
      /// Claim handle (extrinsic)
      const payload_ext = {
        baseHandle: handle_bytes,
        expiration: currentBlock + expirationOffset,
      };
      const claimHandlePayload = ExtrinsicHelper.api.registry.createType(
        'CommonPrimitivesHandlesClaimHandlePayload',
        payload_ext
      );
      const claimHandle = ExtrinsicHelper.claimHandle(msaOwnerKeys, claimHandlePayload);
      const { target: event } = await claimHandle.fundAndSend(fundingSource);
      assert.notEqual(event, undefined, 'claimHandle should return an event');
      const displayHandle = event!.data.handle.toUtf8();
      assert.notEqual(displayHandle, '', 'claimHandle should emit a handle');

      // get handle using msa (rpc)
      const handle_response = await ExtrinsicHelper.getHandleForMSA(msaId);
      if (!handle_response.isSome) {
        throw new Error('handle_response should be Some');
      }
      const full_handle_state = handle_response.unwrap();
      const suffix_from_state = full_handle_state.suffix;
      const suffix = suffix_from_state.toNumber();
      assert.notEqual(suffix, 0, 'suffix should not be 0');
      assert.equal(suffix, suffix_assumed, 'suffix should be equal to suffix_assumed');

      /// Get MSA from full display handle (rpc)
      const msaOption = await ExtrinsicHelper.getMsaForHandle(displayHandle);
      assert(msaOption.isSome, 'msaOption should be Some');
      const msaFromHandle = msaOption.unwrap();
      assert.equal(msaFromHandle.toString(), msaId.toString(), 'msaFromHandle should be equal to msaId');

      // Check that the rpc returns the index as > 0
      const apiCheck = await ExtrinsicHelper.apiPromise.call.handlesRuntimeApi.checkHandle(handle);
      assert(apiCheck.suffixIndex.toNumber() > 0);
    });
  });

  describe('👇 Negative Test: Early retire handle', function () {
    it('should not be able to retire a handle before expiration', async function () {
      const handle_response = await ExtrinsicHelper.getHandleForMSA(msaId);
      if (!handle_response.isSome) {
        throw new Error('handle_response should be Some');
      }

      const full_handle_state = handle_response.unwrap();
      const suffix_from_state = full_handle_state.suffix;
      const suffix = suffix_from_state.toNumber();
      assert.notEqual(suffix, 0, 'suffix should not be 0');

      const currentBlock = await getBlockNumber();
      // Must be at least 6 > original expiration to make sure we get past the finalization
      await ExtrinsicHelper.runToBlock(currentBlock + expirationOffset + 6);
      try {
        const retireHandle = ExtrinsicHelper.retireHandle(msaOwnerKeys);
        const { target: event } = await retireHandle.fundAndSend(fundingSource);
        assert.equal(event, undefined, 'retireHandle should not return an event');
      } catch (e) {
        assert.notEqual(e, undefined, 'retireHandle should throw an error');
      }
    });
  });
});

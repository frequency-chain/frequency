import '@frequency-chain/api-augment';
import assert from 'assert';
import {
  createKeys,
  signPayloadSr25519,
  generateAddKeyPayload,
  createProviderKeysAndId,
  DOLLARS,
  createAndFundKeypairs,
} from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { u64 } from '@polkadot/types';
import { getFundingSource } from '../scaffolding/funding';
import { getUnifiedPublicKey } from '../scaffolding/ethereum';
import { H160 } from '@polkadot/types/interfaces';
import { ethereumAddressToKeyringPair } from '@frequency-chain/ethereum-utils';

const fundingSource = getFundingSource(import.meta.url);

describe('MSA Key management: delete keys and retire', function () {
  let keys: KeyringPair;
  let secondaryKey: KeyringPair;
  let msaId: u64;
  let msaAccountId: H160;

  before(async function () {
    // Generates a msa with two control keys
    // Fund all the different keys
    [keys, secondaryKey] = await createAndFundKeypairs(fundingSource, ['keys', 'secondaryKey'], 2n * DOLLARS);

    const { target } = await ExtrinsicHelper.createMsa(keys).signAndSend();
    assert.notEqual(target?.data.msaId, undefined, 'MSA Id not in expected event');
    msaId = target!.data.msaId;

    // Send tokens to the MSA account
    ({ accountId: msaAccountId } = await ExtrinsicHelper.apiPromise.call.msaRuntimeApi.getEthereumAddressForMsaId(msaId));
    const fundingOp = ExtrinsicHelper.transferFunds(fundingSource, ethereumAddressToKeyringPair(msaAccountId), 1n * DOLLARS);
    const { target: fundingEvent } = await fundingOp.signAndSend();
    assert.notEqual(fundingEvent, undefined, 'should have funded MSA account');

    const payload = await generateAddKeyPayload({
      msaId,
      newPublicKey: getUnifiedPublicKey(secondaryKey),
    });
    const payloadData = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', payload);
    const ownerSig = signPayloadSr25519(keys, payloadData);
    const newSig = signPayloadSr25519(secondaryKey, payloadData);
    const op = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, newSig, payload);
    const { target: event } = await op.signAndSend();
    assert.notEqual(event, undefined, 'should have added public key');

    // Make sure we are finalized before all the tests
    await ExtrinsicHelper.waitForFinalization();
  });

  it('should disallow retiring an MSA with more than one key authorized', async function () {
    const retireOp = ExtrinsicHelper.retireMsa(keys);
    await assert.rejects(retireOp.signAndSend('current'), {
      name: 'RpcError',
      message: /Custom error: 3/,
    });
  });

  it('should fail to delete public key for self', async function () {
    const op = ExtrinsicHelper.deletePublicKey(keys, getUnifiedPublicKey(keys));
    await assert.rejects(op.signAndSend('current'), {
      name: 'RpcError',
      message: /Custom error: 4/,
    });
  });

  it("should fail to delete key if not authorized for key's MSA", async function () {
    const [providerKeys] = await createProviderKeysAndId(fundingSource, 1n * DOLLARS, false);
    const op = ExtrinsicHelper.deletePublicKey(providerKeys, getUnifiedPublicKey(keys));
    await assert.rejects(op.signAndSend('current'), {
      name: 'RpcError',
      message: /Custom error: 5/,
    });
  });

  it("should test for 'NoKeyExists' error", async function () {
    const key = createKeys('nothing key');
    const op = ExtrinsicHelper.deletePublicKey(keys, getUnifiedPublicKey(key));
    await assert.rejects(op.signAndSend('current'), {
      name: 'RpcError',
      message: /Custom error: 1/,
    });
  });

  it('should delete secondary key', async function () {
    const op = ExtrinsicHelper.deletePublicKey(keys, getUnifiedPublicKey(secondaryKey));
    const { target: event } = await op.signAndSend();
    assert.notEqual(event, undefined, 'should have returned PublicKeyDeleted event');
  });

  it('should fail to retire MSA if MSA holds tokens', async function () {
    const retireMsaOp = ExtrinsicHelper.retireMsa(keys);
    await assert.rejects(retireMsaOp.signAndSend('current'), {
      name: 'RpcError',
      message: /Custom error: 2/,
    });
  });

  it('should allow retiring MSA after additional keys have been deleted', async function () {
    const retireMsaOp = ExtrinsicHelper.retireMsa(keys);

    // Make sure we are finalized removing before trying to retire
    await ExtrinsicHelper.waitForFinalization();

    const { target: event, eventMap } = await retireMsaOp.signAndSend('current');

    assert.notEqual(eventMap['msa.PublicKeyDeleted'], undefined, 'should have deleted public key (retired)');
    assert.notEqual(event, undefined, 'should have retired msa');
  });
});

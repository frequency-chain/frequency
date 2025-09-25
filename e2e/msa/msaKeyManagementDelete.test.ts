import '@frequency-chain/api-augment';
import assert from 'assert';
import {
  createAndFundKeypairs,
  createKeys,
  createProviderKeysAndId,
  DOLLARS,
  generateAddKeyPayload,
  generateAuthorizedKeyPayload,
  signPayloadSr25519,
} from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { u64 } from '@polkadot/types';
import { getFundingSource } from '../scaffolding/funding';
import { ethereumAddressToKeyringPair, getUnifiedPublicKey } from '@frequency-chain/ethereum-utils';
import { H160 } from '@polkadot/types/interfaces';

let fundingSource: KeyringPair;
const FUNDS_AMOUNT = 5n * DOLLARS;

describe('MSA Key management: delete keys and retire', function () {
  let keys: KeyringPair;
  let secondaryKey: KeyringPair;
  let msaId: u64;
  let msaAccountId: H160;

  // retiring
  let retiringMsaKey: KeyringPair;
  let retiringMsaId: u64;
  let retiringMsaAccountId: H160;

  before(async function () {
    fundingSource = await getFundingSource(import.meta.url);
    // Generates a msa with two control keys
    // Fund all the different keys
    [keys, secondaryKey] = await createAndFundKeypairs(fundingSource, ['keys', 'secondaryKey'], FUNDS_AMOUNT);
    const { target } = await ExtrinsicHelper.createMsa(keys).signAndSend();
    assert.notEqual(target?.data.msaId, undefined, 'MSA Id not in expected event');
    msaId = target!.data.msaId;

    // Send tokens to the MSA account
    ({ accountId: msaAccountId } =
      await ExtrinsicHelper.apiPromise.call.msaRuntimeApi.getEthereumAddressForMsaId(msaId));
    const fundingOp = ExtrinsicHelper.transferFunds(
      fundingSource,
      ethereumAddressToKeyringPair(msaAccountId),
      3n * DOLLARS
    );
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

    // setup retiring msa and keys
    [retiringMsaKey] = await createAndFundKeypairs(fundingSource, ['retriringMsaKey'], FUNDS_AMOUNT);
    const { target: target2 } = await ExtrinsicHelper.createMsa(retiringMsaKey).signAndSend();
    assert.notEqual(target2?.data.msaId, undefined, 'MSA Id not in expected event');
    retiringMsaId = target2!.data.msaId;

    ({ accountId: retiringMsaAccountId } =
      await ExtrinsicHelper.apiPromise.call.msaRuntimeApi.getEthereumAddressForMsaId(retiringMsaId));
    const fundingOp2 = ExtrinsicHelper.transferFunds(
      fundingSource,
      ethereumAddressToKeyringPair(retiringMsaAccountId),
      3n * DOLLARS
    );
    const { target: fundingEvent2 } = await fundingOp2.signAndSend();
    assert.notEqual(fundingEvent2, undefined, 'should have funded MSA account');

    // Make sure we are finalized before all the tests
    await ExtrinsicHelper.waitForFinalization();
  });

  it('check all different errors for retiring msa', async function () {
    // should disallow retiring an MSA with more than one key authorized
    const retireOp = ExtrinsicHelper.retireMsa(keys);
    await assert.rejects(retireOp.signAndSend('current'), {
      name: 'RpcError',
      message: /Custom error: 3/,
    });

    // should fail to delete public key for self
    const op = ExtrinsicHelper.deletePublicKey(keys, getUnifiedPublicKey(keys));
    await assert.rejects(op.signAndSend('current'), {
      name: 'RpcError',
      message: /Custom error: 4/,
    });

    // should test for 'NoKeyExists' error
    const key2 = createKeys('nothing key');
    const op2 = ExtrinsicHelper.deletePublicKey(keys, getUnifiedPublicKey(key2));
    await assert.rejects(op2.signAndSend('current'), {
      name: 'RpcError',
      message: /Custom error: 1/,
    });

    // should fail to delete key if not authorized for key's MSA
    const [providerKeys1] = await createProviderKeysAndId(fundingSource, 2n * DOLLARS, false);
    const op3 = ExtrinsicHelper.deletePublicKey(providerKeys1, getUnifiedPublicKey(keys));
    await assert.rejects(op3.signAndSend('current'), {
      name: 'RpcError',
      message: /Custom error: 5/,
    });

    // should delete secondary key and should fail to retire MSA if MSA holds tokens
    const op4 = ExtrinsicHelper.deletePublicKey(keys, getUnifiedPublicKey(secondaryKey));
    const { target: event4 } = await op4.signAndSend();
    assert.notEqual(event4, undefined, 'should have returned PublicKeyDeleted event');

    // Make sure we are finalized removing before trying to retire
    await ExtrinsicHelper.waitForFinalization();

    const retireMsaOp = ExtrinsicHelper.retireMsa(keys);
    await assert.rejects(retireMsaOp.signAndSend('current'), {
      name: 'RpcError',
      message: /Custom error: 11/,
    });
  });

  it.skip('should allow retiring MSA after additional keys have been deleted and tokens withdran', async function () {
    // Withdraw tokens from MSA account
    const receiverKeys = createKeys('receiver keys');
    const payload = await generateAuthorizedKeyPayload({
      discriminant: 'AuthorizedKeyData',
      msaId: retiringMsaId,
      authorizedPublicKey: getUnifiedPublicKey(receiverKeys),
    });
    const payloadToSign = ExtrinsicHelper.api.registry.createType('PalletMsaAuthorizedKeyData', payload);
    const ownerSig = signPayloadSr25519(retiringMsaKey, payloadToSign);
    const drainMsaOp = ExtrinsicHelper.withdrawTokens(receiverKeys, retiringMsaKey, ownerSig, payload);
    const { target: withdrawTransferEvent } = await drainMsaOp.signAndSend();
    assert.notEqual(withdrawTransferEvent, undefined, 'should have withdrawn tokens from MSA account');

    const retireMsaOp = ExtrinsicHelper.retireMsa(retiringMsaKey);
    // Make sure we are finalized removing before trying to retire
    await ExtrinsicHelper.waitForFinalization();

    const { target: event, eventMap } = await retireMsaOp.signAndSend('current');

    assert.notEqual(eventMap['msa.PublicKeyDeleted'], undefined, 'should have deleted public key (retired)');
    assert.notEqual(event, undefined, 'should have retired msa');
  });
});

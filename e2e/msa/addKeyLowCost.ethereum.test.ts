import { KeyringPair } from '@polkadot/keyring/types';
import { AddKeyData, Extrinsic, ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import {
  assertEvent,
  CENTS,
  createKeys,
  createMsa,
  createMsaAndProvider,
  DOLLARS,
  generateAddKeyPayload,
  getEthereumKeyPairFromUnifiedAddress,
  signPayloadSr25519,
  stakeToProvider,
} from '../scaffolding/helpers';
import { getUnifiedAddress, getUnifiedPublicKey } from '@frequency-chain/ethereum-utils';
import assert from 'assert';
import { getFundingSource } from '../scaffolding/funding';
import { u64 } from '@polkadot/types';
import { MessageSourceId } from '@frequency-chain/api-augment/interfaces';
import { createAddKeyData, sign } from '@frequency-chain/ethereum-utils';
import { u8aToHex } from '@polkadot/util';

let fundingSource: KeyringPair;

describe('adding an Ethereum key for low cost', function () {
  let providerKeys;
  let providerMsaId;

  before(async function () {
    fundingSource = await getFundingSource(import.meta.url);
    providerKeys = await createKeys('KeyAdder');
    providerMsaId = await createMsaAndProvider(fundingSource, providerKeys, 'KeyAdder', 10n * CENTS);
    await stakeToProvider(fundingSource, fundingSource, providerMsaId, 6n * DOLLARS);
  });

  // create a delegator MSA account with new keys,
  // create a new Ethereum keypair,
  // call generateAddKeyCallParamsUsingKeys,
  // return the new keys, the resulting payload, and both signatures.
  async function generateAddKeyCallParams() {
    const ethereumKeyringPair = await createKeys('Ethereum', 'ethereum');
    const [delegatorMsaId, delegatorKeys] = await createMsa(fundingSource, 10n * CENTS);
    const { addKeyPayload, delegatorSig, newSig } = await generateAddKeyCallParamsUsingKeys(
      delegatorKeys,
      delegatorMsaId,
      ethereumKeyringPair
    );
    return { delegatorKeys, addKeyPayload, delegatorSig, newSig };
  }

  // create AddKeyData using the provided keys.
  // use the keys to sign the AddKeyData in an AddKey payload.
  // return the new keys, the payload, and both signatures.
  async function generateAddKeyCallParamsUsingKeys(
    delegatorKeys: KeyringPair,
    delegatorMsaId: u64,
    ethereumKeyringPair: KeyringPair
  ) {
    const addKeyPayload = await generateAddKeyPayload({});
    addKeyPayload.msaId = delegatorMsaId;
    addKeyPayload.newPublicKey = getUnifiedPublicKey(ethereumKeyringPair);

    const srSignatureaddKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', addKeyPayload);
    const delegatorSrSignature = signPayloadSr25519(delegatorKeys, srSignatureaddKeyData);

    const ethereumSecretKey = u8aToHex(
      getEthereumKeyPairFromUnifiedAddress(getUnifiedAddress(ethereumKeyringPair)).secretKey
    );
    const eip712AddKeyData = createAddKeyData(
      addKeyPayload.msaId.toBigInt(),
      u8aToHex(addKeyPayload.newPublicKey),
      addKeyPayload.expiration
    );
    const ecdsaSignature = await sign(ethereumSecretKey, eip712AddKeyData, 'Dev');

    return { addKeyPayload, delegatorSig: delegatorSrSignature, newSig: ecdsaSignature };
  }

  it('addPublicKeyToMsa costs less for capacity call with eligibility conditions', async function () {
    // SET UP
    const { delegatorKeys, addKeyPayload, delegatorSig, newSig } = await generateAddKeyCallParams();

    // the extrinsic will be called by a provider with stake.
    const addPublicKeyOp = new Extrinsic(
      () =>
        ExtrinsicHelper.api.tx.msa.addPublicKeyToMsa(
          getUnifiedPublicKey(delegatorKeys),
          delegatorSig,
          newSig,
          addKeyPayload
        ),
      providerKeys,
      ExtrinsicHelper.api.events.msa.PublicKeyAdded
    );

    // ACT  pay with capacity using the provider.
    const { eventMap } = await addPublicKeyOp.payWithCapacity();

    // ASSERT it's a very small fee, but not free.
    assertEvent(eventMap, 'msa.PublicKeyAdded');
    const capacityFee = ExtrinsicHelper.getCapacityFee(eventMap);
    assert(capacityFee > 0);
    assert(capacityFee < 1_320_000n); // ~1.3 CENTS

    // add another key; this should cost a lot more
    const thirdKeyEth = await createKeys('Eth2', 'ethereum');
    const delegatorMsaId: MessageSourceId = addKeyPayload.msaId || new u64(ExtrinsicHelper.api.registry, 0);
    const newParams = await generateAddKeyCallParamsUsingKeys(delegatorKeys, delegatorMsaId, thirdKeyEth);

    // again to be submitted by provider.
    const addThirdKeyOp = new Extrinsic(
      () =>
        ExtrinsicHelper.api.tx.msa.addPublicKeyToMsa(
          getUnifiedPublicKey(delegatorKeys),
          newParams.delegatorSig,
          newParams.newSig,
          newParams.addKeyPayload
        ),
      providerKeys,
      ExtrinsicHelper.api.events.msa.PublicKeyAdded
    );

    // ACT pay with capacity to add a third key
    const { eventMap: eventMap2 } = await addThirdKeyOp.payWithCapacity();
    assertEvent(eventMap2, 'msa.PublicKeyAdded');
    const thirdKeyCapacityFee = ExtrinsicHelper.getCapacityFee(eventMap2);
    // 4260363n vs
    // 1278109n
    assert(thirdKeyCapacityFee > capacityFee);
    assert(thirdKeyCapacityFee < 5n * CENTS);
  });
});

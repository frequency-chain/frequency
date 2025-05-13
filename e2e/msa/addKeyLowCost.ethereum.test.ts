import { KeyringPair } from '@polkadot/keyring/types';
import { AddKeyData, EventMap, Extrinsic, ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import {
  assertEvent,
  CENTS,
  createKeys,
  createMsa,
  createMsaAndProvider,
  DOLLARS,
  generateAddKeyPayload,
  getEthereumKeyPairFromUnifiedAddress,
  signEip712AddKeyData,
  signPayloadSr25519,
  stakeToProvider,
} from '../scaffolding/helpers';
import { getUnifiedAddress, getUnifiedPublicKey } from '../scaffolding/ethereum';
import assert from 'assert';
import { getFundingSource } from '../scaffolding/funding';
import { U64, u64 } from '@polkadot/types';
import { BigInt } from '@polkadot/x-bigint';
import { MessageSourceId } from '@frequency-chain/api-augment/interfaces';

const fundingSource = getFundingSource(import.meta.url);

describe('adding an Ethereum key for low cost', function () {
  let providerKeys;
  let providerMsaId;
  const defaultPayload: AddKeyData = {};

  before(async function () {
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
    const ethereumKeyPair = getEthereumKeyPairFromUnifiedAddress(getUnifiedAddress(ethereumKeyringPair));
    const addKeyPayload = await generateAddKeyPayload({});
    addKeyPayload.msaId = delegatorMsaId;
    addKeyPayload.newPublicKey = getUnifiedPublicKey(ethereumKeyringPair);
    const addKeyData = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', addKeyPayload);

    const delegatorSig = signPayloadSr25519(delegatorKeys, addKeyData);
    const newSig = await signEip712AddKeyData(ethereumKeyPair, addKeyPayload);
    return { addKeyPayload, delegatorSig, newSig };
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
    assert(capacityFee < 4n * CENTS);

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
    // 3258176n
    assert(thirdKeyCapacityFee > capacityFee);
    assert(thirdKeyCapacityFee < 5n * CENTS);
  });
});

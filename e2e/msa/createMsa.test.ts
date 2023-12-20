import '@frequency-chain/api-augment';
import assert from 'assert';
import {GetKnownKey, Sr25519Signature, createAndFundKeypair, createKeys, fundKeypair, generateAddKeyPayload, getExistentialDeposit, getNonce, signPayloadSr25519} from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import {AddKeyData, Extrinsic, ExtrinsicHelper} from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';
import {u64} from "@polkadot/types";
import { KNOWN_KEYS } from '../scaffolding/known_keys';
import { Codec } from '@polkadot/types/types';

const fundingSource = getFundingSource('msa-create-msa');

async function createMsa(keys: KeyringPair): Promise<u64> {
  const result = await ExtrinsicHelper.createMsa(keys).signAndSend();
  const msaRecord = result[1]['msa.MsaCreated'];
  if (msaRecord) return msaRecord.data[0] as u64;

  throw 'Failed to get MSA Id...';
}

async function addNewKey(msaOwner :KeyringPair, msaId: u64) {
  const secondaryKey = createKeys();

  let defaultPayload: AddKeyData = {};
  defaultPayload.msaId = msaId;
  defaultPayload.newPublicKey = secondaryKey.publicKey;
  let payload: AddKeyData = await generateAddKeyPayload(defaultPayload);
  
  let addKeyData: Codec = ExtrinsicHelper.api.registry.createType('PalletMsaAddKeyData', payload);

  let ownerSig: Sr25519Signature = signPayloadSr25519(msaOwner, addKeyData);
  let newSig: Sr25519Signature = signPayloadSr25519(secondaryKey, addKeyData);
  const addPublicKeyOp = ExtrinsicHelper.addPublicKeyToMsa(msaOwner, ownerSig, newSig, payload);

  const { target: publicKeyEvents } = await addPublicKeyOp.fundAndSend(fundingSource);

  assert.notEqual(publicKeyEvents, undefined, 'should have added public key');
}

async function retireMsa(msaOwner: KeyringPair) {
  const retireMsaOp = ExtrinsicHelper.retireMsa(msaOwner);
  const { target: event, eventMap } = await retireMsaOp.signAndSend('current');

  assert.notEqual(eventMap['msa.PublicKeyDeleted'], undefined, 'should have deleted public key (retired)');
  assert.notEqual(event, undefined, 'should have retired msa');
}

describe('Create Accounts', function () {
  let keys: KeyringPair;

  before(async function () {
    keys = await createAndFundKeypair(fundingSource, 50_000_000n);
  });

  describe('createMsa', function () {
    it('should successfully create an MSA account', async function () {

      // for( let steps = 0; steps < 4 ; steps++){
      //   try {
      //     const controlKeyPromises: Array<Promise<KeyringPair>> = [];
      //     let devAccountNonce = await getNonce(fundingSource);
      //     const ed = await getExistentialDeposit();
      //     let count = 60;
      //     for (let i = 0; i < count; i++) {
      //       controlKeyPromises.push(createAndFundKeypair(fundingSource, 100n * 10n * ed, undefined, devAccountNonce++));
      //     }
      //     const controlKeys = await Promise.all(controlKeyPromises);

      //     // Create the msas
      //     const msaPromises: Array<Promise<u64>> = [];
      //     for (let i = 0; i < count; i++) {
      //       msaPromises.push(createMsa(controlKeys[i]));
      //     }

      //     const msaIds = await Promise.all(msaPromises);
      //   }catch (ex){
      //     console.error(ex);
      //   }
      // }

      for( let steps = 0; steps < 10 ; steps++){
        const controlKeyPromises: Array<Promise<void>> = [];
        let devAccountNonce = await getNonce(fundingSource);
        const ed = await getExistentialDeposit();
        for (let index = 1 + steps * 100; index <= 100; index++) {
          try {
            let accountKey = GetKnownKey(index - 1);  
            controlKeyPromises.push(fundKeypair(fundingSource, accountKey, 100n * 10n * ed, devAccountNonce++));

          }catch (ex) {
            console.error(ex);
          }
        }
        await Promise.all(controlKeyPromises);

        const addedPromises: Array<Promise<void>> = [];
        for (let index = 1+ steps * 100; index <= 100; index++) {
          try {
            let accountKey = GetKnownKey(index - 1);  
            // addedPromises.push(addNewKey(accountKey, index as unknown as u64));
            addedPromises.push(retireMsa(accountKey));
          }catch (ex) {
            console.error(ex);
          }
        }
        await Promise.all(addedPromises);
      }
    });
  });
});

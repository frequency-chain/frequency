import '@frequency-chain/api-augment';
import assert from 'assert';
import {createAndFundKeypair, getExistentialDeposit, getNonce} from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import {Extrinsic, ExtrinsicHelper} from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';
import {u64} from "@polkadot/types";

const fundingSource = getFundingSource('msa-create-msa');

async function createMsa(keys: KeyringPair): Promise<u64> {
  const result = await ExtrinsicHelper.createMsa(keys).signAndSend();
  const msaRecord = result[1]['msa.MsaCreated'];
  if (msaRecord) return msaRecord.data[0] as u64;

  throw 'Failed to get MSA Id...';
}
describe('Create Accounts', function () {
  let keys: KeyringPair;

  before(async function () {
    keys = await createAndFundKeypair(fundingSource, 50_000_000n);
  });

  describe('createMsa', function () {
    it('should successfully create an MSA account', async function () {

      for( let steps = 0; steps < 4 ; steps++){
        try {
          const controlKeyPromises: Array<Promise<KeyringPair>> = [];
          let devAccountNonce = await getNonce(fundingSource);
          const ed = await getExistentialDeposit();
          let count = 60;
          for (let i = 0; i < count; i++) {
            controlKeyPromises.push(createAndFundKeypair(fundingSource, 100n * 10n * ed, undefined, devAccountNonce++));
          }
          const controlKeys = await Promise.all(controlKeyPromises);

          // Create the msas
          const msaPromises: Array<Promise<u64>> = [];
          for (let i = 0; i < count; i++) {
            msaPromises.push(createMsa(controlKeys[i]));
          }

          const msaIds = await Promise.all(msaPromises);
        }catch (ex){
          console.error(ex);
        }
      }
    });
  });
});

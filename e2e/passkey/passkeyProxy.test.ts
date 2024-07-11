import '@frequency-chain/api-augment';
import assert from 'assert';
import { createAndFundKeypair } from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';

const fundingSource = getFundingSource('passkey-proxy');

describe('Passkey Pallet Tests', function () {
  let fundedKeys: KeyringPair;

  before(async function () {
    fundedKeys = await createAndFundKeypair(fundingSource, 50_000_000n);
  });

  describe('proxy', function () {
  });
});

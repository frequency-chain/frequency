import '@frequency-chain/api-augment';
import assert from 'assert';
import { assertExtrinsicSucceededAndFeesPaid, CENTS, createAndFundKeypair } from '../scaffolding/helpers';
import { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper } from '../scaffolding/extrinsicHelpers';
import { getFundingSource } from '../scaffolding/funding';

const fundingSource = getFundingSource(import.meta.url);

describe('Create Accounts', function () {
  let keys: KeyringPair;

  before(async function () {
    keys = await createAndFundKeypair(fundingSource, 5n * CENTS);
  });

  describe('createMsa', function () {
    it('should successfully create an MSA account', async function () {
      const f = ExtrinsicHelper.createMsa(keys);
      const { target: msaCreatedEvent, eventMap: chainEvents } = await f.fundAndSend(fundingSource);

      assert.notEqual(
        chainEvents['system.ExtrinsicSuccess'],
        undefined,
        'should have returned an ExtrinsicSuccess event'
      );
      assert.notEqual(msaCreatedEvent, undefined, 'should have returned  an MsaCreated event');
      assertExtrinsicSucceededAndFeesPaid(chainEvents);
      assert.notEqual(msaCreatedEvent?.data.msaId, undefined, 'Failed to get the msaId from the event');
    });

    it('should fail to create an MSA for a keypair already associated with an MSA', async function () {
      const op = ExtrinsicHelper.createMsa(keys);
      await assert.rejects(op.fundAndSend(fundingSource), {
        name: 'KeyAlreadyRegistered',
      });
    });
  });
});

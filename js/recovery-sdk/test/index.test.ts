import assert from 'assert';
import {
  getRecoveryCommitment,
  generateRecoverySecret,
  getIntermediaryHashes,
  getRecoveryCommitmentFromIntermediary,
} from '../src/index.js';
import { ContactType } from '../src/types.js';

const MOCK_RECOVERY_SECRET = '69EC-2382-E1E6-76F3-341F-3414-9DD5-CFA5-6932-E418-9385-0358-31DF-AFEA-9828-D3B7';

describe('External Recovery SDK Interface', function () {
  describe('generateRecoverySecret', function () {
    it('should be able to successfully generate a recovery secret in the correct format', function () {
      const recSecret = generateRecoverySecret();
      const expected = /^[A-F0-9]{4}(-[A-F0-9]{4}){15}$/;
      assert.match(recSecret, expected);
    });
  });

  describe('getIntermediaryHashes', function () {
    it('should be able to successfully hash to the intermediary hashes with email', function () {
      const { a, b } = getIntermediaryHashes(MOCK_RECOVERY_SECRET, ContactType.EMAIL, 'user@example.com');
      assert.equal(a, '0xb7537721c183333b4fc18fe5cf20dd4718a9d867fa4800ea3b266a0d3e601c16');
      assert.equal(b, '0x157f274caaba6f60f76eca3d308d8aae5f9a6771072e0eec0ad4f925fa9ce15e');
    });
  });

  describe('getRecoveryCommitment', function () {
    it('should be able to successfully hash to the recovery commitment with email', function () {
      const commitment = getRecoveryCommitment(MOCK_RECOVERY_SECRET, ContactType.EMAIL, 'user@example.com');
      assert.equal(commitment, '0xafd940c7321ee6983ecb8fae3bb0919d6ea5830734cbe95808dab0a43f8ac4cc');
    });
  });

  describe('getRecoveryCommitmentFromIntermediary', function () {
    it('should be able to successfully hash to the recovery commitment from the Intermediary Hashes', function () {
      const commitment = getRecoveryCommitmentFromIntermediary(
        '0xb7537721c183333b4fc18fe5cf20dd4718a9d867fa4800ea3b266a0d3e601c16',
        '0x157f274caaba6f60f76eca3d308d8aae5f9a6771072e0eec0ad4f925fa9ce15e'
      );
      assert.equal(commitment, '0xafd940c7321ee6983ecb8fae3bb0919d6ea5830734cbe95808dab0a43f8ac4cc');
    });
  });
});

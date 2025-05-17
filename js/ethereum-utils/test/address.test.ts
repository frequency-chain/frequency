import assert from 'assert';
import {
  reverseUnifiedAddressToEthereumAddress,
  HexString,
  getSS58AccountFromEthereumAccount,
  getUnifiedPublicKey,
  getUnifiedAddress,
} from '../src';
import { hexToU8a, u8aToHex } from '@polkadot/util';

describe('Address tests', function () {
  it('should correctly extract the Ethereum address from a valid unified address', function () {
    // Arrange
    const validEthAddress = '0x1234567890123456789012345678901234567890' as HexString;
    const unifiedAddress = `${validEthAddress}${'ee'.repeat(12)}` as HexString;

    // Act
    const result = reverseUnifiedAddressToEthereumAddress(unifiedAddress);

    // Assert
    assert.deepEqual(result, validEthAddress);
  });

  it('should throw an error when the unified address is not reversible', function () {
    // Arrange
    const invalidUnifiedAddress = '0x1234567890123456789012345678901234567890abcdef' as HexString;

    // Act & Assert
    assert.throws(
      () => {
        reverseUnifiedAddressToEthereumAddress(invalidUnifiedAddress);
      },
      new Error(`Address ${invalidUnifiedAddress} is not reversible!`)
    );
  });

  it('should correctly convert an Ethereum address to SS58 format', function () {
    const ethereumAddress = '0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac';
    const ss58 = '5HYRCKHYJN9z5xUtfFkyMj4JUhsAwWyvuU8vKB1FcnYTf9ZQ';

    const result = getSS58AccountFromEthereumAccount(ethereumAddress);

    assert.equal(result, ss58);
  });

  describe('getUnifiedPublicKey ', function () {
    it('should return the original publicKey for sr25519 and ed25519', function () {
      // Arrange
      const originalPublicKey = new Uint8Array(32).fill(3); // Dummy public key
      const ed25519Pair = {
        type: 'ed25519',
        publicKey: originalPublicKey,
      };

      // Act
      const result = getUnifiedPublicKey(ed25519Pair);

      // Assert
      assert.equal(result, originalPublicKey);

      // Also test with sr25519 type
      const sr25519Pair = {
        type: 'sr25519',
        publicKey: originalPublicKey,
      };

      const result2 = getUnifiedPublicKey(sr25519Pair);
      assert.equal(result2, originalPublicKey);
    });

    it('should throw an error for ecdsa key type', function () {
      // Arrange
      const ecdsaPair = {
        type: 'ecdsa',
        publicKey: new Uint8Array(32).fill(2),
      };

      // Act & Assert
      assert.throws(() => {
        getUnifiedPublicKey(ecdsaPair);
      }, new Error('Ecdsa key type is not supported and it should be replaced with ethereum ones!'));
    });

    it('should properly handle ethereum key type', function () {
      const ethPublicKey = '0x02509540919faacf9ab52146c9aa40db68172d83777250b28e4679176e49ccdd9f';
      const ethPair = {
        type: 'ethereum',
        publicKey: hexToU8a(ethPublicKey),
      };
      const unifiedPublicKey = hexToU8a('0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566caceeeeeeeeeeeeeeeeeeeeeeee');

      const result = getUnifiedPublicKey(ethPair);
      assert.deepEqual(result, unifiedPublicKey);
    });
  });

  describe('getUnifiedAddress ', function () {
    it('should properly handle ethereum key type', function () {
      const ethPublicKey = '0x02509540919faacf9ab52146c9aa40db68172d83777250b28e4679176e49ccdd9f';
      const ethPair = {
        type: 'ethereum',
        publicKey: hexToU8a(ethPublicKey),
      };
      const unifiedPublicKey = '5HYRCKHYJN9z5xUtfFkyMj4JUhsAwWyvuU8vKB1FcnYTf9ZQ';

      const result = getUnifiedAddress(ethPair);
      assert.deepEqual(result, unifiedPublicKey);
    });
  });
});

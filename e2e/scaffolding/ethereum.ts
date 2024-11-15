/* eslint-disable no-restricted-syntax */
import { KeyringPair } from '@polkadot/keyring/types';
import { encodeAddress, ethereumEncode } from '@polkadot/util-crypto';
import { hexToU8a, u8aToHex } from '@polkadot/util';
import type { Signer } from '@polkadot/types/types';
import { SignerResult } from '@polkadot/types/types';
import { secp256k1 } from '@noble/curves/secp256k1';
import { Keyring } from '@polkadot/api';
import { Keypair } from '@polkadot/util-crypto/types';
import { Address20MultiAddress } from './helpers';

/**
 * Returns unified 32 bytes SS58 accountId
 * @param pair
 */
export function getUnifiedAddress(pair: KeyringPair): string {
  if ('ethereum' === pair.type) {
    const etheAddressHex = ethereumEncode(pair.publicKey);
    return getSS58AccountFromEthereumAccount(etheAddressHex);
  }
  if (pair.type === 'ecdsa') {
    throw new Error(`ecdsa type is not supported!`);
  }
  return pair.address;
}

/**
 * Returns ethereum style public key with prefixed zeros example: 0x00000000000000000000000019a701d23f0ee1748b5d5f883cb833943096c6c4
 * @param pair
 */
export function getUnifiedPublicKey(pair: KeyringPair): Uint8Array {
  if ('ethereum' === pair.type) {
    const publicKeyBytes = hexToU8a(ethereumEncode(pair.publicKey));
    const result = new Uint8Array(32);
    result.fill(0, 0, 12);
    result.set(publicKeyBytes, 12);
    return result;
  }
  if (pair.type === 'ecdsa') {
    throw new Error(`ecdsa type is not supported!`);
  }
  return pair.publicKey;
}

/**
 * This custom signer can get used to mimic EIP-191 message signing. By replacing the `ethereumPair.sign` with
 * any wallet call we can sign any extrinsic with any wallet
 * @param ethereumPair
 */
export function getEthereumStyleSigner(ethereumPair: KeyringPair): Signer {
  return {
    signRaw: async (payload): Promise<SignerResult> => {
      const sig = ethereumPair.sign(prefixEthereumTags(payload.data));
      const prefixedSignature = new Uint8Array(sig.length + 1);
      prefixedSignature[0] = 2;
      prefixedSignature.set(sig, 1);
      const hex = u8aToHex(prefixedSignature);
      return {
        signature: hex,
      } as SignerResult;
    },
  };
}

/**
 * Convert a keyPair into a 20 byte ethereum address
 * @param pair
 */
export function getAccountId20MultiAddress(pair: KeyringPair): Address20MultiAddress {
  if (pair.type !== "ethereum") {
    throw new Error(`Only ethereum keys are supported!`);
  }
  const etheAddress = ethereumEncode(pair.publicKey);
  const ethAddress20 = Array.from(hexToU8a(etheAddress));
  return { Address20: ethAddress20 };
}

/**
 *
 * @param secretKey of secp256k1 keypair exported from any wallet (should be 32 bytes)
 */
export function getKeyringPairFromSecp256k1PrivateKey(secretKey: Uint8Array): KeyringPair {
  const publicKey = secp256k1.getPublicKey(secretKey, true);
  const keypair: Keypair = {
    secretKey,
    publicKey,
  };
  const keyring = new Keyring({ type: 'ethereum' });
  return keyring.addFromPair(keypair, undefined, 'ethereum');
}

/**
 * converts an ethereum account to SS58 format
 * @param accountId20Hex
 */
function getSS58AccountFromEthereumAccount(accountId20Hex: string): string {
  const addressBytes = hexToU8a(accountId20Hex);
  const result = new Uint8Array(32);
  result.fill(0, 0, 12);
  result.set(addressBytes, 12);
  return encodeAddress(result);
}

/**
 * This is a helper method to allow being able to create a signature that might be created by Metamask
 * @param hexPayload
 */
function wrapCustomFrequencyTag(hexPayload: string): Uint8Array {
  // wrapping in frequency tags to show this is a Frequency related payload
  const frequencyWrapped = `<Frequency>${hexPayload.toLowerCase()}</Frequency>`;
  return prefixEthereumTags(frequencyWrapped);
}

/**
 * prefixing with the EIP-191 for personal_sign messages (this gets wrapped automatically in metamask)
 * @param hexPayload
 */
function prefixEthereumTags(hexPayload: string): Uint8Array {
  const wrapped = `\x19Ethereum Signed Message:\n${hexPayload.length}${hexPayload}`;
  const buffer = Buffer.from(wrapped, 'utf-8');
  return new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.length);
}

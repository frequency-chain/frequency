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
import { H160 } from '@polkadot/types/interfaces';

/**
 * Create a partial KeyringPair from an Ethereum address
 */
export function ethereumAddressToKeyringPair(ethereumAddress: H160): KeyringPair {
  return {
    type: 'ethereum',
    address: ethereumAddress.toHex(),
    addressRaw: ethereumAddress,
  } as unknown as KeyringPair;
}

/**
 * Returns unified 32 bytes SS58 accountId
 * @param pair
 */
export function getUnifiedAddress(pair: KeyringPair): string {
  if ('ethereum' === pair.type) {
    const etheAddressHex = ethereumEncode(pair.publicKey || pair.address);
    return getSS58AccountFromEthereumAccount(etheAddressHex);
  }
  if (pair.type === 'ecdsa') {
    throw new Error('Ecdsa key type is not supported and it should be replaced with ethereum ones!');
  }
  return pair.address;
}

/**
 * Returns ethereum style public key with suffixed 0xee example: 0x19a701d23f0ee1748b5d5f883cb833943096c6c4eeeeeeeeeeeeeeeeeeeeeeee
 * @param pair
 */
export function getUnifiedPublicKey(pair: KeyringPair): Uint8Array {
  if ('ethereum' === pair.type) {
    const ethAddressBytes = hexToU8a(ethereumEncode(pair.publicKey));
    const suffix = new Uint8Array(12).fill(0xee);
    const result = new Uint8Array(32);
    result.set(ethAddressBytes, 0);
    result.set(suffix, 20);
    return result;
  }
  if (pair.type === 'ecdsa') {
    throw new Error('Ecdsa key type is not supported and it should be replaced with ethereum ones!');
  }
  return pair.publicKey;
}

export function reverseUnifiedAddressToEthereumAddress(unifiedAddress: Uint8Array) : String {
  let hex = Buffer.from(unifiedAddress).toString("hex");
  if (!hex.toLowerCase().endsWith("eeeeeeeeeeeeeeeeeeeeeeee")) {
    throw new Error(`Address ${hex} is not reversible!`);
  }
  return `0x${hex.substring(0, 40)}`;
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
  if (pair.type !== 'ethereum') {
    throw new Error(`Only ethereum keys are supported!`);
  }
  const etheAddress = ethereumEncode(pair.publicKey || pair.address);
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
  return new Keyring({ type: 'ethereum' }).createFromPair(keypair, undefined, 'ethereum');
}

/**
 * converts an ethereum account to SS58 format
 * @param accountId20Hex
 */
export function getSS58AccountFromEthereumAccount(accountId20Hex: string): string {
  const addressBytes = hexToU8a(accountId20Hex);
  const suffix = new Uint8Array(12).fill(0xee);
  const result = new Uint8Array(32);
  result.set(addressBytes, 0);
  result.set(suffix, 20);
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
 * prefixing with the EIP-191 for personal_sign messages (this gets wrapped automatically in Metamask)
 * @param hexPayload
 */
function prefixEthereumTags(hexPayload: string): Uint8Array {
  const wrapped = `\x19Ethereum Signed Message:\n${hexPayload.length}${hexPayload}`;
  const buffer = Buffer.from(wrapped, 'utf-8');
  return new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.length);
}

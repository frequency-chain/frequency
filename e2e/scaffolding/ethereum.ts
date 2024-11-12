import { KeyringPair } from '@polkadot/keyring/types';
import { encodeAddress, ethereumEncode } from '@polkadot/util-crypto';
import { hexToU8a, u8aToHex } from '@polkadot/util';
import type { Signer } from '@polkadot/types/types';
import { SignerResult } from '@polkadot/types/types';
import { secp256k1 } from '@noble/curves/secp256k1';
import { Keyring } from '@polkadot/api';
import { Keypair } from '@polkadot/util-crypto/types';
import { Address20MultiAddress } from './helpers';

export function getUnifiedAddress(pair: KeyringPair): string {
  if ('ethereum' === pair.type) {
    const etheAddressHex = ethereumEncode(pair.publicKey);
    return getConvertedEthereumAccount(etheAddressHex);
  }
  if (pair.type === 'ecdsa') {
    throw new Error(`ecdsa type is not supported!`);
  }
  return pair.address;
}

export function getEthereumStyleSigner(ethereumPair: KeyringPair): Signer {
  return {
    signRaw: async (payload): Promise<SignerResult> => {
      console.log(`raw_payload: ${payload.data}`);
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
 * This is a helper method to allow being able to create a signature that might be created by metamask
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
  console.log(`wrapped ${wrapped}`);
  const buffer = Buffer.from(wrapped, 'utf-8');
  return new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.length);
}

export function getAccountId20MultiAddress(pair: KeyringPair): Address20MultiAddress {
  const etheAddress = ethereumEncode(pair.publicKey);
  const ethAddress20 = Array.from(hexToU8a(etheAddress));
  return { Address20: ethAddress20 };
}

/**
 * Returns ethereum style public key with prefixed zeros example: 0x00000000000000000000000019a701d23f0ee1748b5d5f883cb833943096c6c4
 * @param pair
 */
export function getConvertedEthereumPublicKey(pair: KeyringPair): Uint8Array {
  const publicKeyBytes = hexToU8a(ethereumEncode(pair.publicKey));
  const result = new Uint8Array(32);
  result.fill(0, 0, 12);
  result.set(publicKeyBytes, 12);
  return result;
}

function getConvertedEthereumAccount(accountId20Hex: string): string {
  const addressBytes = hexToU8a(accountId20Hex);
  const result = new Uint8Array(32);
  result.fill(0, 0, 12);
  result.set(addressBytes, 12);
  return encodeAddress(result);
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

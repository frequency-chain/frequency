import { H160 } from '@polkadot/types/interfaces';
import { KeyringPair } from '@polkadot/keyring/types';
import { encodeAddress, ethereumEncode } from '@polkadot/util-crypto';
import { hexToU8a, u8aToHex } from '@polkadot/util';
import { Address20MultiAddress, EthereumKeyPair, HexString } from './payloads.js';
import { ethers } from 'ethers';
import { Keyring } from '@polkadot/api';
import { Keypair } from '@polkadot/util-crypto/types';

/**
 * Creates a Random Ethereum key
 */
export function createRandomKey(): EthereumKeyPair {
  const k = ethers.Wallet.createRandom();
  return {
    publicKey: k.publicKey,
    privateKey: k.privateKey,
    address: {
      ethereumAddress: k.address,
      unifiedAddress: getUnified32BytesAddress(k.address),
      unifiedAddressSS58: getSS58AccountFromEthereumAccount(k.address),
    },
    mnemonic: k.mnemonic?.phrase ?? '',
  } as EthereumKeyPair;
}

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
    const unifiedHex = getUnified32BytesAddress(u8aToHex(pair.publicKey));
    return hexToU8a(unifiedHex);
  }
  if (pair.type === 'ecdsa') {
    throw new Error('Ecdsa key type is not supported and it should be replaced with ethereum ones!');
  }
  return pair.publicKey;
}

export function reverseUnifiedAddressToEthereumAddress(unifiedAddress: HexString): HexString {
  if (!unifiedAddress.toLowerCase().endsWith('ee'.repeat(12))) {
    throw new Error(`Address ${unifiedAddress} is not reversible!`);
  }
  return `${unifiedAddress.substring(0, 42)}` as HexString;
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
 *
 * @param secretKey of secp256k1 keypair exported from any wallet (should be 32 bytes)
 */
export function getKeyringPairFromSecp256k1PrivateKey(secretKey: Uint8Array): KeyringPair {
  const publicKey = ethers.SigningKey.computePublicKey(secretKey, true);
  const keypair: Keypair = {
    secretKey,
    publicKey: hexToU8a(publicKey),
  };
  return new Keyring({ type: 'ethereum' }).createFromPair(keypair, undefined, 'ethereum');
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

function getUnified32BytesAddress(ethAddressOrPublicKey: string): HexString {
  const ethAddressBytes = hexToU8a(ethereumEncode(ethAddressOrPublicKey));
  const suffix = new Uint8Array(12).fill(0xee);
  const result = new Uint8Array(32);
  result.set(ethAddressBytes, 0);
  result.set(suffix, 20);
  return u8aToHex(result);
}

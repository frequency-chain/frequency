/**
 * The AutoNonce Module keeps track of nonces used by tests
 * Because all keys in the tests are managed by the tests, the nonces are determined by:
 * 1. Prior transaction count
 * 2. Not counting transactions that had RPC failures
 */

import type { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper } from './extrinsicHelpers';
import {getUnifiedAddress} from "./ethereum";

export type AutoNonce = number | 'auto' | 'current';

const nonceCache = new Map<string, number>();

const getNonce = async (keys: KeyringPair) => {
  return (await ExtrinsicHelper.getAccountInfo(getUnifiedAddress(keys))).nonce.toNumber();
};

const reset = (keys: KeyringPair) => {
  nonceCache.delete(getUnifiedAddress(keys));
};

const current = async (keys: KeyringPair): Promise<number> => {
  return nonceCache.get(getUnifiedAddress(keys)) || (await getNonce(keys));
};

const increment = async (keys: KeyringPair) => {
  const nonce = await current(keys);
  nonceCache.set(getUnifiedAddress(keys), nonce + 1);
  return nonce;
};

/**
 * Use the auto nonce system
 * @param keys The KeyringPair that will be used for sending a transaction
 * @param inputNonce
 *        "auto" (default) for using the auto system
 *        "current" for just getting the current one instead of incrementing
 *        <number> for just using a specific number (also sets it to increments for the future)
 * @returns
 */
const auto = (keys: KeyringPair, inputNonce: AutoNonce = 'auto'): Promise<number> => {
  switch (inputNonce) {
    case 'auto':
      return increment(keys);
    case 'current':
      return current(keys);
    default:
      nonceCache.set(getUnifiedAddress(keys), inputNonce + 1);
      return Promise.resolve(inputNonce);
  }
};

export default {
  auto,
  reset,
};

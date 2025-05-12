/**
 * The AutoNonce Module keeps track of nonces used by tests
 * Because all keys in the tests are managed by the tests, the nonces are determined by:
 * 1. Prior transaction count
 * 2. Not counting transactions that had RPC failures
 */

import type { KeyringPair } from '@polkadot/keyring/types';
import { ExtrinsicHelper } from './extrinsicHelpers';
import { getUnifiedAddress } from './ethereum';

export type AutoNonce = number | 'auto' | 'current';

// Fixed size mapping for addresses. Using a power of 2 for efficient modulo
const ADDRESSES_MASK = 0xff; // 256 addresses
// Create a dictionary from address to index
const addressIndices = new Map<string, number>();
let nextAddressIndex = 0;

// Create shared buffer for nonce storage
// Format: [addressCount, addressIndex1, nonce1, addressIndex2, nonce2, ...]
// This allows us to store both nonces and maintain address mapping atomically
const MAX_ENTRIES = ADDRESSES_MASK + 1;
const HEADER_SIZE = 1; // Just storing address count for now
const ENTRY_SIZE = 2; // Each entry has index and nonce
const BUFFER_SIZE = HEADER_SIZE + MAX_ENTRIES * ENTRY_SIZE;

const sharedBuffer = new SharedArrayBuffer(Int32Array.BYTES_PER_ELEMENT * BUFFER_SIZE);
const sharedArray = new Int32Array(sharedBuffer);

// Initialize the header
Atomics.store(sharedArray, 0, 0); // No addresses yet

const getSlotOffset = (slot: number) => HEADER_SIZE + slot * ENTRY_SIZE;

// Function to claim a slot for a new address
const claimAddressSlot = (address: string): number => {
  // First check if we already have this address
  if (addressIndices.has(address)) {
    return addressIndices.get(address)!;
  }

  // Get next available index atomically
  const index = Atomics.add(sharedArray, 0, 1);
  if (index >= MAX_ENTRIES) {
    throw new Error(`autoNonce ERROR: Exceeded maximum number of addresses (${MAX_ENTRIES})`);
  }

  // Store in our local map
  const slot = index % MAX_ENTRIES;
  addressIndices.set(address, slot);

  const slotOffset = getSlotOffset(slot);
  // Use a unique identifier for this address
  const addressId = nextAddressIndex++;
  Atomics.store(sharedArray, slotOffset, addressId);
  Atomics.store(sharedArray, slotOffset + 1, 0); // Initial nonce = 0

  return slot;
};

const getAddressNonce = (address: string): number => {
  if (!addressIndices.has(address)) {
    return 0;
  }

  const slot = addressIndices.get(address)!;
  const slotOffset = getSlotOffset(slot);

  return Atomics.load(sharedArray, slotOffset + 1);
};

const setAddressNonce = (address: string, nonce: number): void => {
  const slot = addressIndices.has(address) ? addressIndices.get(address)! : claimAddressSlot(address);
  const slotOffset = getSlotOffset(slot);

  Atomics.store(sharedArray, slotOffset + 1, nonce);
};

const incrementAddressNonce = (address: string): number => {
  const slot = addressIndices.has(address) ? addressIndices.get(address)! : claimAddressSlot(address);
  const slotOffset = getSlotOffset(slot);

  return Atomics.add(sharedArray, slotOffset + 1, 1);
};

const getNonce = async (keys: KeyringPair) => {
  return (await ExtrinsicHelper.getAccountInfo(keys)).nonce.toNumber();
};

const reset = (keys: KeyringPair) => {
  const address = getUnifiedAddress(keys);
  if (addressIndices.has(address)) {
    setAddressNonce(address, 0);
  }
};

// Manage concurrent blockchain queries
const pendingQueries = new Map<string, Promise<number>>();

const current = async (keys: KeyringPair): Promise<number> => {
  const address = getUnifiedAddress(keys);

  // If we already have a query in progress, return that
  if (pendingQueries.has(address)) {
    return pendingQueries.get(address)!;
  }

  // Check for stored nonce
  const storedNonce = getAddressNonce(address);
  if (storedNonce > 0) {
    return storedNonce;
  }

  // Otherwise, we need to fetch from the blockchain
  const promise = getNonce(keys).then((nonce) => {
    setAddressNonce(address, nonce);
    pendingQueries.delete(address);
    return nonce;
  });

  pendingQueries.set(address, promise);
  return promise;
};

const increment = async (keys: KeyringPair) => {
  const address = getUnifiedAddress(keys);
  await current(keys);
  return incrementAddressNonce(address);
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
      // Use a manual nonce if it is a number
      const address = getUnifiedAddress(keys);
      setAddressNonce(address, inputNonce + 1);
      return Promise.resolve(inputNonce);
  }
};

export default {
  auto,
  reset,
};

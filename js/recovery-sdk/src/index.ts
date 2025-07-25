import { keccak_256 } from '@noble/hashes/sha3';
import { bytesToHex, randomBytes, concatBytes, hexToBytes, utf8ToBytes } from '@noble/hashes/utils';
import { ContactType, HexString } from './types.js';
import { standardizeContact } from './standardize.js';

export * from './types.js';

/**
 * Generates a new Recovery Secret for a user
 *
 * @returns A new recovery secret string formatted for user consumption
 */
export function generateRecoverySecret(): string {
  const unformatted = bytesToHex(randomBytes(32)).split('');
  return [...Array(64 / 4)]
    .map((_a) => unformatted.splice(0, 4).join(''))
    .join('-')
    .toUpperCase();
}

/**
 * Gets the Intermediary Hashes `a` and `b` used to do the Recovery
 *
 * @param recoverySecret The string form of the Recovery Secret
 * @param contactType Email = "0x00", Phone = "0x01"
 * @param contact The raw contact string
 * @returns { a: Hex String of H(s), b: H(s + sc) }
 */
export function getIntermediaryHashes(
  recoverySecret: string,
  contactType: ContactType,
  contact: string
): { a: HexString; b: HexString } {
  const { a, b } = getIntermediaryHashesAsUint8Array(recoverySecret, contactType, contact);
  return {
    a: `0x${bytesToHex(a)}`,
    b: `0x${bytesToHex(b)}`,
  };
}

function getIntermediaryHashesAsUint8Array(
  recoverySecret: string,
  contactType: ContactType,
  contact: string
): { a: Uint8Array; b: Uint8Array } {
  const s = hexToBytes(recoverySecret.replaceAll('-', ''));
  const contactStandard = standardizeContact(contactType, contact);

  return {
    a: keccak_256(s),
    b: keccak_256(concatBytes(s, hexToBytes(contactType.replace('0x', '')), utf8ToBytes(contactStandard))),
  };
}

/**
 * Gets the Recovery Commitment that is stored on chain for the validation at time of recovery
 *
 * @param recoverySecret The string form of the Recovery Secret
 * @param contactType Email = "0x00", Phone = "0x01"
 * @param contact The raw contact string
 * @returns the hex string of the Recovery Commitment
 */
export function getRecoveryCommitment(recoverySecret: string, contactType: ContactType, contact: string): HexString {
  const { a, b } = getIntermediaryHashesAsUint8Array(recoverySecret, contactType, contact);

  return `0x${bytesToHex(keccak_256(concatBytes(a, b)))}`;
}

/**
 * Gets the Recovery Commitment that is stored on chain for the validation at time of recovery from the Intermediary Hashes
 *
 * @param aHash A hash, H(s)
 * @param bHash B hash, H(s + c)
 * @returns the hex string of the Recovery Commitment
 */
export function getRecoveryCommitmentFromIntermediary(aHash: string, bHash: string): HexString {
  const a = hexToBytes(aHash.replace('0x', ''));
  const b = hexToBytes(bHash.replace('0x', ''));
  return `0x${bytesToHex(keccak_256(concatBytes(a, b)))}`;
}

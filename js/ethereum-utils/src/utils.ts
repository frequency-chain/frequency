import { HexString } from './payloads.js';

/**
 * Validate that a number is a valid uint16 (0 to 65535)
 */
export function isValidUint16(value: number): boolean {
  return Number.isInteger(value) && value >= 0 && value <= 65535;
}

/**
 * Validate that a number is a valid uint32 (0 to 4294967295)
 */
export function isValidUint32(value: number): boolean {
  return Number.isInteger(value) && value >= 0 && value <= 4294967295;
}

/**
 * Validate that a number is a valid uint64 (0 to 18446744073709551615n)
 */
export function isValidUint64String(value: bigint | string): boolean {
  const bigIntValue = typeof value === 'string' ? BigInt(value) : value;
  return bigIntValue >= 0 && bigIntValue <= 18446744073709551615n;
}

/**
 * Validate that a string is a valid hex string
 */
export function isHexString(value: string): value is HexString {
  // Check if string starts with '0x' and contains only hex characters
  const hexRegex = /^0[xX][0-9a-fA-F]*$/;
  const isHex = hexRegex.test(value);
  return isHex && value.length % 2 === 0;
}

/**
 * Universal assert function
 * @param condition
 * @param message
 */
export function assert(condition: boolean, message?: string): asserts condition {
  if (!condition) {
    throw new Error(message || ' Assertion failed');
  }
}

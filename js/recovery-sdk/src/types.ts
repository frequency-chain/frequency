/**
 * The contact types supported
 */
export enum ContactType {
  /**
   * Email is standardized and utf8 encoded and appended after the type
   */
  EMAIL = '0x00',
  /**
   * Phone is standardized as E.164 and utf8 encoded and appended after the type
   */
  PHONE = '0x01',
}

/**
 * Identifies a 0x-prefixed base16 string
 */
export type HexString = `0x${string}`;

import { parsePhoneNumberFromString } from 'libphonenumber-js/min';
import { ContactType } from './types.js';

/**
 * Cleans up the email by lowercasing and removing periods in the username
 * Does NOT validate the email
 *
 * @param email Raw email
 * @returns standardized email string
 */
function standardizeEmail(email: string): string {
  return email.toLowerCase().trim();
}

/**
 * Parses and standardizes the phone number to the E.164 Standard
 *
 * @param phone Raw phone number. Assumed to be a US phone number if not complete
 * @returns
 */
function standardizePhone(phone: string): string {
  const parsed = parsePhoneNumberFromString(phone, 'US');
  if (!parsed) throw new Error('Unable to parse contact method');
  return parsed.format('E.164');
}

/**
 * Converts the contact into the standard form for hashing
 *
 * @param contactType Email = "0x00", Phone = "0x01"
 * @param contact The raw contact string
 * @returns The standardized string ready for conversion to utf8 bytes and hashed
 */
export function standardizeContact(contactType: ContactType, contact: string): string {
  switch (contactType) {
    case ContactType.EMAIL:
      return standardizeEmail(contact);
    case ContactType.PHONE:
      return standardizePhone(contact);
  }
}

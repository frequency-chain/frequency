import assert from 'assert';
import { standardizeContact } from '../src/standardize.js';
import { ContactType } from '../src/types.js';

describe('standardizeContact', function () {
  it('should be able to successfully standardize an email', function () {
    const contact = standardizeContact(ContactType.EMAIL, 'LoWERcase-no-d.o.t.s@LOWERCASE.Co.uK');
    assert.equal(contact, 'lowercase-no-dots@lowercase.co.uk');
  });

  it('should trim all the whitespace in the email username and domain', function () {
    const contact = standardizeContact(ContactType.EMAIL, '   a   @   b.com  ');
    assert.equal(contact, 'a@b.com');
  });

  it('should be able to successfully standardize a basic US phone', function () {
    const contact = standardizeContact(ContactType.PHONE, '1-800-8675309');
    assert.equal(contact, '+18008675309');
  });

  it('should be able to successfully standardize some various phone numbers', function () {
    assert.equal(standardizeContact(ContactType.PHONE, '+44 20-7946--0958x123'), '+442079460958');
    assert.equal(standardizeContact(ContactType.PHONE, '1 (213) 373 4253'), '+12133734253');
    assert.equal(standardizeContact(ContactType.PHONE, '+49-0176.123.4567abc'), '+491761234567');
  });
});

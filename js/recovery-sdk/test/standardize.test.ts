import assert from 'assert';
import { standardizeContact } from '../src/standardize.js';
import { ContactType } from '../src/types.js';

describe('standardizeContact', function () {
  it('should be able to successfully standardize an email', function () {
    const contact = standardizeContact(ContactType.EMAIL, 'LoWERcase.it@LOWERCASE.Co.uK');
    assert.equal(contact, 'lowercase.it@lowercase.co.uk');
  });

  it('should trim the email', function () {
    const contact = standardizeContact(ContactType.EMAIL, '   a@b.com  ');
    assert.equal(contact, 'a@b.com');
  });

  it('should throw for an invalid email', function () {
    assert.throws(() => standardizeContact(ContactType.EMAIL, 'more@than@once@example.com'), Error);
    assert.throws(
      () =>
        standardizeContact(
          ContactType.EMAIL,
          'this_is_longer_than_is_allowed_by_the_email_spec__this_is_longer_than_is_allowed_by_the_email_spec__this_is_longer_than_is_allowed_by_the_email_spec__this_is_longer_than_is_allowed_by_the_email_spec__this_is_longer_than_is_allowed_by_the_email_spec@example.com'
        ),
      Error
    );
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

  it('should throw for an invalid phone', function () {
    assert.throws(() => standardizeContact(ContactType.PHONE, '867-5309-867-5309-867-5309'), Error);
  });
});

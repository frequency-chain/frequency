import assert from 'assert';
import { FULL_NAME_TO_ID, ID_TO_SCHEMA_FULL_NAME, ID_TO_SCHEMA_INFO, SchemaInfo } from '../schemas';

describe('schemas', function () {
  it('should be able to successfully get schemas from Id', function () {
    const fullName = ID_TO_SCHEMA_FULL_NAME.get(7);
    assert.equal(fullName, 'dsnp.public-key-key-agreement@v1');
  });

  it('should be able to successfully get schema id from name', function () {
    const id = FULL_NAME_TO_ID.get('dsnp.public-key-key-agreement@v1');
    assert.equal(id, 7);
  });

  it('should be able to successfully get schema info from ID', function () {
    const info = ID_TO_SCHEMA_INFO.get(7);
    const expected: SchemaInfo = {
      id: 7,
      namespace: 'dsnp',
      name: 'public-key-key-agreement',
      version: 1,
      deprecated: false,
      modelType: 'AvroBinary',
      payloadLocation: 'Itemized',
      appendOnly: true,
      signatureRequired: true,
    };
    assert.equal(info.id, expected.id);
    assert.equal(info.namespace, expected.namespace);
    assert.equal(info.name, expected.name);
    assert.equal(info.version, expected.version);
    assert.equal(info.deprecated, expected.deprecated);
    assert.equal(info.modelType, expected.modelType);
    assert.equal(info.payloadLocation, expected.payloadLocation);
    assert.equal(info.appendOnly, expected.appendOnly);
    assert.equal(info.signatureRequired, expected.signatureRequired);
  });
});

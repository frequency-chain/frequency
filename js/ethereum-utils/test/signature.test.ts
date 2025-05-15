import assert from 'assert';
import {
  createAddKeyData,
  createAddProvider,
  createClaimHandlePayload,
  createItemizedAddAction,
  createItemizedDeleteAction,
  createItemizedSignaturePayloadV2,
  createPaginatedDeleteSignaturePayloadV2,
  createPaginatedUpsertSignaturePayloadV2,
  createPasskeyPublicKey,
} from '../src';

describe('Signature Payload Creation tests', function () {
  it('should create valid createAddKeyData payloads', function () {
    const payload1 = createAddKeyData('27', '0x123456', 90);
    const payload2 = createAddKeyData(27n, new Uint8Array([18, 52, 86]), 90);

    const expected = {
      type: 'AddKeyData',
      msaId: 27,
      expiration: 90,
      newPublicKey: '0x123456',
    };

    assert.deepEqual(payload1, expected);
    assert.deepEqual(payload2, expected);
  });

  it('should create valid createAddProvider payloads', function () {
    const payload1 = createAddProvider('27', [18, 52, 86], 90);
    const payload2 = createAddProvider(27n, [18, 52, 86], 90);

    const expected = {
      type: 'AddProvider',
      authorizedMsaId: 27,
      expiration: 90,
      schemaIds: [18, 52, 86],
    };

    assert.deepEqual(payload1, expected);
    assert.deepEqual(payload2, expected);
  });

  it('should create valid createClaimHandlePayload payloads', function () {
    const payload1 = createClaimHandlePayload('test', 90);

    const expected = {
      type: 'ClaimHandlePayload',
      handle: 'test',
      expiration: 90,
    };

    assert.deepEqual(payload1, expected);
  });

  it('should create valid createPasskeyPublicKey payloads', function () {
    const payload1 = createPasskeyPublicKey('0x123456');
    const payload2 = createPasskeyPublicKey(new Uint8Array([18, 52, 86]));

    const expected = {
      type: 'PasskeyPublicKey',
      publicKey: '0x123456',
    };

    assert.deepEqual(payload1, expected);
    assert.deepEqual(payload2, expected);
  });

  it('should create valid createItemizedSignaturePayloadV2 payloads', function () {
    const addAction = createItemizedAddAction('0x123456');
    const addAction2 = createItemizedAddAction(new Uint8Array([18, 52, 86]));
    const deleteAction = createItemizedDeleteAction(4);
    const actions = [addAction, addAction2, deleteAction];
    const payload1 = createItemizedSignaturePayloadV2(64, 92187389, 90, actions);

    const expected = {
      type: 'ItemizedSignaturePayloadV2',
      schemaId: 64,
      targetHash: 92187389,
      expiration: 90,
      actions,
    };

    assert.deepEqual(payload1, expected);
  });

  it('should create valid createPaginatedDeleteSignaturePayloadV2 payloads', function () {
    const payload1 = createPaginatedDeleteSignaturePayloadV2(1, 2, 34324, 90);

    const expected = {
      type: 'PaginatedDeleteSignaturePayloadV2',
      schemaId: 1,
      pageId: 2,
      targetHash: 34324,
      expiration: 90,
    };

    assert.deepEqual(payload1, expected);
  });

  it('should create valid createPaginatedUpsertSignaturePayloadV2 payloads', function () {
    const payload1 = createPaginatedUpsertSignaturePayloadV2(1, 2, 34324, 90, '0x123456');
    const payload2 = createPaginatedUpsertSignaturePayloadV2(1, 2, 34324, 90, new Uint8Array([18, 52, 86]));

    const expected = {
      type: 'PaginatedUpsertSignaturePayloadV2',
      schemaId: 1,
      pageId: 2,
      targetHash: 34324,
      expiration: 90,
      payload: '0x123456',
    };

    assert.deepEqual(payload1, expected);
    assert.deepEqual(payload2, expected);
  });
});

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
  signEip712,
  HexString,
  verifyEip712Signature,
  createSiwfSignedRequest,
  createSiwfLoginRequestPayload,
} from '../src';

describe('Signature related tests', function () {
  describe('Signature Payload Creation tests', function () {
    it('should create valid createAddKeyData payloads', function () {
      const payload1 = createAddKeyData('27', '0x123456', 90);
      const payload2 = createAddKeyData(27n, new Uint8Array([18, 52, 86]), 90);

      const expected = {
        type: 'AddKeyData',
        msaId: '27',
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
        authorizedMsaId: '27',
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

  describe('EIP-712 Signing tests', function () {
    const secretKey: HexString = '0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133';

    it('should create a valid signature for PaginatedUpsertSignaturePayloadV2', async function () {
      const payload1 = createPaginatedUpsertSignaturePayloadV2(
        10,
        5,
        1982672367,
        100,
        '0x0000f20f0efece05af6bee4fe6b24db6febe6988d24f62015e23d1f1ce9b7040301a8a3cefc00c1162067f70d004ed86588cd6183b3564b9d92e9924cba2244a9e9f85e334e1b8bb5e8f770a81866c4c406cfadaef2a8f93bf8b89a5f4efce86a3cfc1ff4d93a8d362d630bf16ac5766de75c5fef57af69b147cb629ad8c1359691c9ab6f5380527e3c18d71b0480747a9628693b86473f1e7f01abe5bb7eadb76fad361669241e74bffed13b49e5c7dd08a8d3a7722bcbfad494627256c7e77f2c9b2b596a5f468a82e06f5905d13707355e2dd77d9b9480ec959a7f870b68fce122ddad81e06fe4dec48e01206df9e6e3a4e0d928b722220ec199d22324fb2ce6a8b6581175476d940bd499435091c0b0b3d0303e3bf60732ff8e898223a39aa8623cf33a1623af2cfc8d6fc83f4b53ed32ef4fdab29035150d9dc3b7e60ac91fb1fbb6895a007913fee3940c8f8e3ececcf50b3076b65c62f8bd8c039f90182d89bf4c0a2f426e957a8732a9936f8a0aab8b3c18183eb605f5a0bac859eb7d6d88eea4c02444116a6dec0a790d83abce10de20bb29c3f980864eb23f555422ff36421e14df39bb53606cbb4dda195c321ce4fa4fc7143ac267ae811b6949d51f521ddaf08a3663dd3a1eb2b783b31dcbfc830f6f542499857767c39fb85a29ebf175e0f0877e89a9e564307a7d7eb9d1f8401f0a65382cfc051fa5d34829381f624e2a556e2c3002eea63e50ad8463bf8ac6096249983f8d1925c0d9392e67203be98daae7305b52962b95fbeca76c7db2ceec6208d21efb68350aedec3a48cba0c9112c93efb98d363dfa26471b163b05d1655f0af7d867fd25dcc4d5e1a5cb3934586ba7a418f489439f3f551c1017bde009dba49dbc132c7066eecea25e2e231491f136a1fe9a83f0f1091c7e9b2cf8d24541fe21af50a46e6537f80ed2065da8842b928cb27ec23169c129863d5540f05a4dac070f3c834cfe998503f4a42a7ae4e2b1ef694e7600027e48721560d8d66a46c5ac7899d71985c7ccb60d4225cd08307c497138b6cbb00803cf9be1251798c6ddd8c972f78455a43acc7bf6278dda25946dc8638c20042ba333ee6e6da605bb05b1d01c5715c1723a2d6147eca637f6a7f50b807476e482c29769e1b94b43ac7b1921e99c60c795cf034a706c28befbcbe448f926ca212eb4607157dd9dd30c89b9a885202789b2ac9a6e030c1d87d5341c59b64105a88caa13cdcc42bbdda752169234e39d94f6aadcfdd99e80e9eb3a10ea5a80ba4825370ad9935d5b8cda568d2c7db63a8e016cb78bd0d657f5d1f6916fb48a3678d973dc8835a49f5e2a0ff07698c360492f568def07397129d290a86cefd0524c826b73e85c48a8e525aafe77bd0c1e9a0c2a4a4ea96343adfe81fad9ef12200ed90c6c476906d710e16d3af77a4e18164'
      );

      const signature = await signEip712(secretKey, payload1);

      const expected = {
        Ecdsa:
          '0xbb182602012c1489a6b98af9d867d7c2c0ef111a1a20653a028e49d4ec60e2a64e4983270c6d9de76eaed3283a3f34a5829920e057b77bed6861b2616c22be381b',
      };

      assert.deepEqual(signature, expected);
      assert(
        verifyEip712Signature('0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac', expected.Ecdsa, payload1),
        'should verify'
      );
    });

    it('should create a valid signature for PaginatedDeleteSignaturePayloadV2', async function () {
      const payload1 = createPaginatedDeleteSignaturePayloadV2(10, 5, 1982672367, 100);
      const signature = await signEip712(secretKey, payload1);

      const expected = {
        Ecdsa:
          '0xd6f327427488e9f03bda92113a9d1c2e881bc3e8d1d6a065b727c154733983b3059ad4fa5cf28cdf1aae9e0faa3fde6427f92686cf55c3a12180610cb3effe371b',
      };

      assert.deepEqual(signature, expected);
      assert(
        verifyEip712Signature('0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac', expected.Ecdsa, payload1),
        'should verify'
      );
    });

    it('should create a valid signature for ItemizedSignaturePayloadV2', async function () {
      const addAction = createItemizedAddAction('0x40a6836ea489047852d3f0297f8fe8ad6779793af4e9c6274c230c207b9b825026');
      const deleteAction = createItemizedDeleteAction(2);
      const actions = [addAction, deleteAction];
      const payload1 = createItemizedSignaturePayloadV2(10, 1982672367, 100, actions);
      const signature = await signEip712(secretKey, payload1);

      const expected = {
        Ecdsa:
          '0xc6c38093c57cd605ca5adfe5d538be89c7bbea4309c797078ed23fe4bb6ad8cb245ae12b19bb9c9b8c084b356b02f6f09f7e107c262c1aff0714a681bc3af5b51b',
      };

      assert.deepEqual(signature, expected);
      assert(
        verifyEip712Signature('0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac', expected.Ecdsa, payload1),
        'should verify'
      );
    });

    it('should create a valid signature for AddKeyData', async function () {
      const payload1 = createAddKeyData(
        12876327n,
        '0x7A23f8d62589aB9651722C7f4a0e998d7d3eF2A9eeeeeeeeeeeeeeeeeeeeeeee',
        100
      );
      const signature = await signEip712(secretKey, payload1);

      const expected = {
        Ecdsa:
          '0x7fb9df5e7f51875509456fe24de92c256c4dcaaaeb952fe36bb30f79c8cc3bbf2f988fa1c55efb6bf20825e98de5cc1ac0bdcf036ad1e0f9ee969a729540ff8d1c',
      };

      assert.deepEqual(signature, expected);
      assert(
        verifyEip712Signature('0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac', expected.Ecdsa, payload1),
        'should verify'
      );
    });

    it('should create a valid signature for AddProvider', async function () {
      const payload1 = createAddProvider(12876327n, [2, 4, 5, 6, 7, 8], 100);
      const signature = await signEip712(secretKey, payload1);

      const expected = {
        Ecdsa:
          '0x34ed5cc291815bdc7d95b418b341bbd3d9ca82c284d5f22d8016c27bb9d4eef8507cdb169a40e69dc5d7ee8ff0bff29fa0d8fc4e73cad6fc9bf1bf076f8e0a741c',
      };

      assert.deepEqual(signature, expected);
      assert(
        verifyEip712Signature('0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac', expected.Ecdsa, payload1),
        'should verify'
      );
    });

    it('should create a valid signature for ClaimHandlePayload', async function () {
      const payload1 = createClaimHandlePayload('Alice', 100);
      const signature = await signEip712(secretKey, payload1);

      const expected = {
        Ecdsa:
          '0x832d1f6870118f5fc6e3cc314152b87dc452bd607581f16b1e39142b553260f8397e80c9f7733aecf1bd46d4e84ad333c648e387b069fa93b4b1ca4fa0fd406b1c',
      };

      assert.deepEqual(signature, expected);
      assert(
        verifyEip712Signature('0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac', expected.Ecdsa, payload1),
        'should verify'
      );
    });

    it('should create a valid signature for PasskeyPublicKey', async function () {
      const payload1 = createPasskeyPublicKey('0x40a6836ea489047852d3f0297f8fe8ad6779793af4e9c6274c230c207b9b825026');
      const signature = await signEip712(secretKey, payload1);

      const expected = {
        Ecdsa:
          '0xbafaf5e21695a502b2d356b4558da35245aa1be7161f01a5f0224fbfdf85b5c52898fc495ab1ca9b68c3b07e23d31a5fe1686165344b22bc14201f293d54f36b1b',
      };

      assert.deepEqual(signature, expected);
      assert(
        verifyEip712Signature('0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac', expected.Ecdsa, payload1),
        'should verify'
      );
    });

    it('should create a valid signature for SiwfSignedRequest', async function () {
      const payload1 = createSiwfSignedRequest('https://localhost:44181', [2, 4, 5, 6, 7, 8]);
      const signature = await signEip712(secretKey, payload1);

      const expected = {
        Ecdsa:
          '0x68b18a8da75fcbddd62d53916173d636129b219f703aa4edb3b0dea51d61c0da606624f69012485d9b361cb0ddd3370f8e3c0b1b80249ddcca8300df1daa0f111c',
      };

      assert.deepEqual(signature, expected);
      assert(
        verifyEip712Signature('0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac', expected.Ecdsa, payload1),
        'should verify'
      );
    });

    it('should create a valid signature for SiwfLoginRequestPayload', async function () {
      const payload1 = createSiwfLoginRequestPayload(
        'your-app.com wants you to sign in with your Frequency account:\n' +
          'f6akufkq9Lex6rT8RCEDRuoZQRgo5pWiRzeo81nmKNGWGNJdJ\n' +
          '\n' +
          '\n' +
          '\n' +
          'URI: https://your-app.com/signin/callback\n' +
          'Nonce: N6rLwqyz34oUxJEXJ\n' +
          'Issued At: 2024-10-29T19:17:27.077Z\n' +
          'Expiration Time: 2060-03-05T23:23:03.041Z'
      );
      const signature = await signEip712(secretKey, payload1);

      const expected = {
        Ecdsa:
          '0x4b428bc8cfce1469fab3196a4271033851c5e604b16a5945c117df35147709476767f118d9d75909a867346dd56f7d1539be1ae69b3b28aaa4ad790689ceb6fa1c',
      };

      assert.deepEqual(signature, expected);
      assert(
        verifyEip712Signature('0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac', expected.Ecdsa, payload1),
        'should verify'
      );
    });
  });
});

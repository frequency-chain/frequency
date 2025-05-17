import { EipDomainPayload } from './payloads';

export const EIP712_DOMAIN_DEFINITION = {
  EIP712Domain: [
    {
      name: 'name',
      type: 'string',
    },
    {
      name: 'version',
      type: 'string',
    },
    {
      name: 'chainId',
      type: 'uint256',
    },
    {
      name: 'verifyingContract',
      type: 'address',
    },
  ],
};

export const EIP712_DOMAIN_DEFAULT: EipDomainPayload = {
  name: 'Frequency',
  version: '1',
  chainId: '0x190f1b44',
  verifyingContract: '0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC',
};

export const ADD_PROVIDER_DEFINITION = {
  AddProvider: [
    {
      name: 'authorizedMsaId',
      type: 'uint64',
    },
    {
      name: 'schemaIds',
      type: 'uint16[]',
    },
    {
      name: 'expiration',
      type: 'uint32',
    },
  ],
};

export const ADD_KEY_DATA_DEFINITION = {
  AddKeyData: [
    {
      name: 'msaId',
      type: 'uint64',
    },
    {
      name: 'expiration',
      type: 'uint32',
    },
    {
      name: 'newPublicKey',
      type: 'address',
    },
  ],
};

export const CLAIM_HANDLE_PAYLOAD_DEFINITION = {
  ClaimHandlePayload: [
    {
      name: 'handle',
      type: 'string',
    },
    {
      name: 'expiration',
      type: 'uint32',
    },
  ],
};

export const PASSKEY_PUBLIC_KEY_DEFINITION = {
  PasskeyPublicKey: [
    {
      name: 'publicKey',
      type: 'bytes',
    },
  ],
};

export const PAGINATED_DELETE_SIGNATURE_PAYLOAD_DEFINITION = {
  PaginatedDeleteSignaturePayloadV2: [
    {
      name: 'schemaId',
      type: 'uint16',
    },
    {
      name: 'pageId',
      type: 'uint16',
    },
    {
      name: 'targetHash',
      type: 'uint32',
    },
    {
      name: 'expiration',
      type: 'uint32',
    },
  ],
};

export const PAGINATED_UPSERT_SIGNATURE_PAYLOAD_DEFINITION = {
  PaginatedUpsertSignaturePayloadV2: [
    {
      name: 'schemaId',
      type: 'uint16',
    },
    {
      name: 'pageId',
      type: 'uint16',
    },
    {
      name: 'targetHash',
      type: 'uint32',
    },
    {
      name: 'expiration',
      type: 'uint32',
    },
    {
      name: 'payload',
      type: 'bytes',
    },
  ],
};

export const ITEMIZED_SIGNATURE_PAYLOAD_DEFINITION = {
  ItemizedSignaturePayloadV2: [
    {
      name: 'schemaId',
      type: 'uint16',
    },
    {
      name: 'targetHash',
      type: 'uint32',
    },
    {
      name: 'expiration',
      type: 'uint32',
    },
    {
      name: 'actions',
      type: 'ItemAction[]',
    },
  ],
  ItemAction: [
    { name: 'actionType', type: 'string' },
    { name: 'data', type: 'bytes' },
    { name: 'index', type: 'uint16' },
  ],
};

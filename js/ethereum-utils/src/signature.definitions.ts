import { EipDomainPayload } from './payloads.js';

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

// using pallet_revive test chain ID for now.
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

export const AUTHORIZED_KEY_DATA_DEFINITION = {
  AuthorizedKeyData: [
    {
      name: 'msaId',
      type: 'uint64',
    },
    {
      name: 'expiration',
      type: 'uint32',
    },
    {
      name: 'authorizedPublicKey',
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

export const PAGINATED_DELETE_SIGNATURE_PAYLOAD_DEFINITION_V2 = {
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

export const PAGINATED_UPSERT_SIGNATURE_PAYLOAD_DEFINITION_V2 = {
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

export const ITEMIZED_SIGNATURE_PAYLOAD_DEFINITION_V2 = {
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

export const SIWF_SIGNED_REQUEST_PAYLOAD_DEFINITION = {
  SiwfSignedRequest: [
    {
      name: 'callback',
      type: 'string',
    },
    {
      name: 'permissions',
      type: 'uint16[]',
    },
    {
      name: 'userIdentifierAdminUrl',
      type: 'string',
    },
  ],
};

const PAYLOAD_DEFINITIONS = [
  ADD_PROVIDER_DEFINITION,
  ADD_KEY_DATA_DEFINITION,
  AUTHORIZED_KEY_DATA_DEFINITION,
  CLAIM_HANDLE_PAYLOAD_DEFINITION,
  PASSKEY_PUBLIC_KEY_DEFINITION,
  PAGINATED_DELETE_SIGNATURE_PAYLOAD_DEFINITION_V2,
  PAGINATED_UPSERT_SIGNATURE_PAYLOAD_DEFINITION_V2,
  ITEMIZED_SIGNATURE_PAYLOAD_DEFINITION_V2,
  SIWF_SIGNED_REQUEST_PAYLOAD_DEFINITION,
];

export type SupportedPayloadDefinitions = (typeof PAYLOAD_DEFINITIONS)[number];

import {
  AddKeyData,
  AuthorizedKeyData,
  AddProvider,
  ChainType,
  ClaimHandlePayload,
  EcdsaSignature,
  ItemizedSignaturePayloadV2,
  PaginatedDeleteSignaturePayloadV2,
  PaginatedUpsertSignaturePayloadV2,
  PasskeyPublicKey,
  SupportedPayload,
  HexString,
  AddItemizedAction,
  DeleteItemizedAction,
  ItemizedAction,
  EipDomainPayload,
  NormalizedSupportedPayload,
  SupportedPayloadTypes,
} from './payloads';
import { assert, isHexString, isValidUint16, isValidUint32, isValidUint64String } from './utils';
import { reverseUnifiedAddressToEthereumAddress } from './address';
import { ethers } from 'ethers';
import { u8aToHex } from '@polkadot/util';
import {
  ADD_KEY_DATA_DEFINITION,
  ADD_PROVIDER_DEFINITION,
  AUTHORIZED_KEY_DATA_DEFINITION,
  CLAIM_HANDLE_PAYLOAD_DEFINITION,
  EIP712_DOMAIN_DEFAULT,
  EIP712_DOMAIN_DEFINITION,
  ITEMIZED_SIGNATURE_PAYLOAD_DEFINITION_V2,
  PAGINATED_DELETE_SIGNATURE_PAYLOAD_DEFINITION_V2,
  PAGINATED_UPSERT_SIGNATURE_PAYLOAD_DEFINITION_V2,
  PASSKEY_PUBLIC_KEY_DEFINITION,
  SupportedPayloadDefinitions,
} from './signature.definitions';

/**
 * Signing EIP-712 compatible signature for payload
 * @param secretKey
 * @param payload
 * @param chain
 */
export async function signEip712(
  secretKey: HexString,
  payload: SupportedPayload,
  chain: ChainType = 'Mainnet-Frequency'
): Promise<EcdsaSignature> {
  const types = getTypesFor(payload.type);
  const normalizedPayload = normalizePayload(payload);
  const wallet = new ethers.Wallet(secretKey);
  // TODO: use correct chainID for different networks
  // TODO: use correct contract address for different payloads
  const signature = await wallet.signTypedData(EIP712_DOMAIN_DEFAULT, types, normalizedPayload);
  return { Ecdsa: signature } as EcdsaSignature;
}

/**
 * Verify EIP-712 signatures
 * @param ethereumAddress
 * @param signature
 * @param payload
 * @param chain
 */
export function verifyEip712Signature(
  ethereumAddress: HexString,
  signature: HexString,
  payload: SupportedPayload,
  chain: ChainType = 'Mainnet-Frequency'
): boolean {
  const types = getTypesFor(payload.type);
  const normalizedPayload = normalizePayload(payload);
  // TODO: use correct chainID for different networks
  // TODO: use correct contract address for different payloads
  const recoveredAddress = ethers.verifyTypedData(EIP712_DOMAIN_DEFAULT, types, normalizedPayload, signature);
  return recoveredAddress.toLowerCase() === ethereumAddress.toLowerCase();
}

function normalizePayload(payload: SupportedPayload): NormalizedSupportedPayload {
  const clonedPayload: typeof payload = Object.assign({}, payload);
  switch (clonedPayload.type) {
    case 'PaginatedUpsertSignaturePayloadV2':
    case 'PaginatedDeleteSignaturePayloadV2':
    case 'ItemizedSignaturePayloadV2':
    case 'PasskeyPublicKey':
    case 'ClaimHandlePayload':
    case 'AddProvider':
      break;

    case 'AddKeyData':
      // convert to 20 bytes ethereum address for signature
      if (clonedPayload.newPublicKey.length !== 42) {
        clonedPayload.newPublicKey = reverseUnifiedAddressToEthereumAddress((payload as AddKeyData).newPublicKey);
      }
      clonedPayload.newPublicKey = clonedPayload.newPublicKey.toLowerCase() as HexString;
      break;

    case 'AuthorizedKeyData':
      // convert to 20 bytes ethereum address for signature
      if (clonedPayload.authorizedPublicKey.length !== 42) {
        clonedPayload.authorizedPublicKey = reverseUnifiedAddressToEthereumAddress((payload as AuthorizedKeyData).authorizedPublicKey);
      }
      clonedPayload.authorizedPublicKey = clonedPayload.authorizedPublicKey.toLowerCase() as HexString;
      break;


    default:
      throw new Error(`Unsupported payload type: ${JSON.stringify(payload)}`);
  }

  // Remove the type field
  const { type, ...payloadWithoutType } = clonedPayload;

  return payloadWithoutType;
}

function getTypesFor(payloadType: string): SupportedPayloadDefinitions {
  const PAYLOAD_TYPE_DEFINITIONS: Record<string, SupportedPayloadDefinitions> = {
    PaginatedUpsertSignaturePayloadV2: PAGINATED_UPSERT_SIGNATURE_PAYLOAD_DEFINITION_V2,
    PaginatedDeleteSignaturePayloadV2: PAGINATED_DELETE_SIGNATURE_PAYLOAD_DEFINITION_V2,
    ItemizedSignaturePayloadV2: ITEMIZED_SIGNATURE_PAYLOAD_DEFINITION_V2,
    PasskeyPublicKey: PASSKEY_PUBLIC_KEY_DEFINITION,
    ClaimHandlePayload: CLAIM_HANDLE_PAYLOAD_DEFINITION,
    AddKeyData: ADD_KEY_DATA_DEFINITION,
    AuthorizedKeyData: AUTHORIZED_KEY_DATA_DEFINITION,
    AddProvider: ADD_PROVIDER_DEFINITION,
  };

  const definition = PAYLOAD_TYPE_DEFINITIONS[payloadType];

  if (!definition) {
    throw new Error(`Unsupported payload type: ${payloadType}`);
  }

  return definition;
}

/**
 * Build an AddKeyData payload for signature.
 *
 * @param msaId           MSA ID (uint64) to add the key
 * @param newPublicKey    32 bytes public key to add in hex or Uint8Array
 * @param expirationBlock Block number after which this payload is invalid
 */
export function createAddKeyData(
  msaId: string | bigint,
  newPublicKey: HexString | Uint8Array,
  expirationBlock: number
): AddKeyData {
  const parsedMsaId: string = typeof msaId === 'string' ? msaId : `${msaId}`;
  const parsedNewPublicKey: HexString = typeof newPublicKey === 'object' ? u8aToHex(newPublicKey) : newPublicKey;

  assert(isValidUint64String(parsedMsaId), 'msaId should be a valid uint64');
  assert(isValidUint32(expirationBlock), 'expiration should be a valid uint32');
  assert(isHexString(parsedNewPublicKey), 'newPublicKey should be valid hex');
  return {
    type: 'AddKeyData',
    msaId: parsedMsaId,
    expiration: expirationBlock,
    newPublicKey: parsedNewPublicKey,
  };
}

/**
 * Build an AuthorizedKeyData payload for signature.
 *
 * @param msaId                  MSA ID (uint64) to add the key
 * @param authorizedPublicKey    32 bytes public key to authorize in hex or Uint8Array
 * @param expirationBlock        Block number after which this payload is invalid
 */
export function createAuthorizedKeyData(
  msaId: string | bigint,
  newPublicKey: HexString | Uint8Array,
  expirationBlock: number
): AuthorizedKeyData {
  const parsedMsaId: string = typeof msaId === 'string' ? msaId : `${msaId}`;
  const parsedNewPublicKey: HexString = typeof newPublicKey === 'object' ? u8aToHex(newPublicKey) : newPublicKey;

  assert(isValidUint64String(parsedMsaId), 'msaId should be a valid uint64');
  assert(isValidUint32(expirationBlock), 'expiration should be a valid uint32');
  assert(isHexString(parsedNewPublicKey), 'newPublicKey should be valid hex');
  return {
    type: 'AuthorizedKeyData',
    msaId: parsedMsaId,
    expiration: expirationBlock,
    authorizedPublicKey: parsedNewPublicKey,
  };
}

/**
 * Build an AddProvider payload for signature.
 *
 * @param authorizedMsaId MSA ID (uint64) that will be granted provider rights
 * @param schemaIds       One or more schema IDs (uint16) the provider may use
 * @param expirationBlock Block number after which this payload is invalid
 */
export function createAddProvider(
  authorizedMsaId: string | bigint,
  schemaIds: number[],
  expirationBlock: number
): AddProvider {
  assert(isValidUint64String(authorizedMsaId), 'authorizedMsaId should be a valid uint64');
  assert(isValidUint32(expirationBlock), 'expiration should be a valid uint32');
  schemaIds.forEach((schemaId) => {
    assert(isValidUint16(schemaId), 'schemaId should be a valid uint16');
  });

  return {
    type: 'AddProvider',
    authorizedMsaId: authorizedMsaId.toString(),
    schemaIds,
    expiration: expirationBlock,
  };
}

/**
 * Build a ClaimHandlePayload for signature.
 *
 * @param handle          The handle the user wishes to claim
 * @param expirationBlock Block number after which this payload is invalid
 */
export function createClaimHandlePayload(handle: string, expirationBlock: number): ClaimHandlePayload {
  assert(handle.length > 0, 'handle should be a valid string');
  assert(isValidUint32(expirationBlock), 'expiration should be a valid uint32');

  return {
    type: 'ClaimHandlePayload',
    handle,
    expiration: expirationBlock,
  };
}

/**
 * Build a PasskeyPublicKey payload for signature.
 *
 * @param publicKey The passkey’s public key (hex string or raw bytes)
 */
export function createPasskeyPublicKey(publicKey: HexString | Uint8Array): PasskeyPublicKey {
  const parsedNewPublicKey: HexString = typeof publicKey === 'object' ? u8aToHex(publicKey) : publicKey;
  assert(isHexString(parsedNewPublicKey), 'publicKey should be valid hex');

  return {
    type: 'PasskeyPublicKey',
    publicKey: parsedNewPublicKey,
  };
}

export function createItemizedAddAction(data: HexString | Uint8Array): AddItemizedAction {
  const parsedData: HexString = typeof data === 'object' ? u8aToHex(data) : data;
  assert(isHexString(parsedData), 'itemized data should be valid hex');
  return { actionType: 'Add', data, index: 0 } as AddItemizedAction;
}

export function createItemizedDeleteAction(index: number): DeleteItemizedAction {
  assert(isValidUint16(index), 'itemized index should be a valid uint16');

  return { actionType: 'Delete', data: '0x', index };
}

/**
 * Build an ItemizedSignaturePayloadV2 for signing.
 *
 * @param schemaId   uint16 schema identifier
 * @param targetHash uint32 page hash
 * @param expiration uint32 expiration block
 * @param actions    Array of Add/Delete itemized actions
 */
export function createItemizedSignaturePayloadV2(
  schemaId: number,
  targetHash: number,
  expiration: number,
  actions: ItemizedAction[]
): ItemizedSignaturePayloadV2 {
  assert(isValidUint16(schemaId), 'schemaId should be a valid uint16');
  assert(isValidUint32(targetHash), 'targetHash should be a valid uint32');
  assert(isValidUint32(expiration), 'expiration should be a valid uint32');
  assert(actions.length > 0, 'At least one action is required for ItemizedSignaturePayloadV2');

  return {
    type: 'ItemizedSignaturePayloadV2',
    schemaId,
    targetHash,
    expiration,
    actions,
  };
}

/**
 * Build a PaginatedDeleteSignaturePayloadV2 for signing.
 *
 * @param schemaId   uint16 schema identifier
 * @param pageId     uint16 page identifier
 * @param targetHash uint32 page hash
 * @param expiration uint32 expiration block
 */
export function createPaginatedDeleteSignaturePayloadV2(
  schemaId: number,
  pageId: number,
  targetHash: number,
  expiration: number
): PaginatedDeleteSignaturePayloadV2 {
  assert(isValidUint16(schemaId), 'schemaId should be a valid uint16');
  assert(isValidUint16(pageId), 'pageId should be a valid uint16');
  assert(isValidUint32(targetHash), 'targetHash should be a valid uint32');
  assert(isValidUint32(expiration), 'expiration should be a valid uint32');

  return {
    type: 'PaginatedDeleteSignaturePayloadV2',
    schemaId,
    pageId,
    targetHash,
    expiration,
  };
}

/**
 * Build a PaginatedUpsertSignaturePayloadV2 for signing.
 *
 * @param schemaId   uint16 schema identifier
 * @param pageId     uint16 page identifier
 * @param targetHash uint32 page hash
 * @param expiration uint32 expiration block
 * @param payload    HexString or Uint8Array data to upsert
 */
export function createPaginatedUpsertSignaturePayloadV2(
  schemaId: number,
  pageId: number,
  targetHash: number,
  expiration: number,
  payload: HexString | Uint8Array
): PaginatedUpsertSignaturePayloadV2 {
  const parsedPayload: HexString = typeof payload === 'object' ? u8aToHex(payload) : payload;

  assert(isValidUint16(schemaId), 'schemaId should be a valid uint16');
  assert(isValidUint16(pageId), 'pageId should be a valid uint16');
  assert(isValidUint32(targetHash), 'targetHash should be a valid uint32');
  assert(isValidUint32(expiration), 'expiration should be a valid uint32');
  assert(isHexString(parsedPayload), 'payload should be valid hex');

  return {
    type: 'PaginatedUpsertSignaturePayloadV2',
    schemaId,
    pageId,
    targetHash,
    expiration,
    payload: parsedPayload,
  };
}

/**
 * Returns the EIP-712 browser request for a AddKeyData for signing.
 *
 * @param msaId           MSA ID (uint64) to add the key
 * @param newPublicKey    32 bytes public key to add in hex or Uint8Array
 * @param expirationBlock Block number after which this payload is invalid
 * @param domain
 */
export function getEip712BrowserRequestAddKeyData(
  msaId: string | bigint,
  newPublicKey: HexString | Uint8Array,
  expirationBlock: number,
  domain: EipDomainPayload = EIP712_DOMAIN_DEFAULT
): unknown {
  const message = createAddKeyData(msaId, newPublicKey, expirationBlock);
  const normalized = normalizePayload(message);
  return createEip712Payload(ADD_KEY_DATA_DEFINITION, message.type, domain, normalized);
}

/**
 * Returns the EIP-712 browser request for a AuthorizedKeyData for signing.
 *
 * @param msaId           MSA ID (uint64) to add the key
 * @param authorizedPublicKey    32 bytes public key to add in hex or Uint8Array
 * @param expirationBlock Block number after which this payload is invalid
 * @param domain
 */
export function getEip712BrowserRequestAuthorizedKeyData(
  msaId: string | bigint,
  authorizedPublicKey: HexString | Uint8Array,
  expirationBlock: number,
  domain: EipDomainPayload = EIP712_DOMAIN_DEFAULT
): unknown {
  const message = createAuthorizedKeyData(msaId, authorizedPublicKey, expirationBlock);
  const normalized = normalizePayload(message);
  return createEip712Payload(AUTHORIZED_KEY_DATA_DEFINITION, message.type, domain, normalized);
}

/**
 * Returns the EIP-712 browser request for a AddProvider for signing.
 *
 * @param authorizedMsaId MSA ID (uint64) that will be granted provider rights
 * @param schemaIds       One or more schema IDs (uint16) the provider may use
 * @param expirationBlock Block number after which this payload is invalid
 * @param domain
 */
export function getEip712BrowserRequestAddProvider(
  authorizedMsaId: string | bigint,
  schemaIds: number[],
  expirationBlock: number,
  domain: EipDomainPayload = EIP712_DOMAIN_DEFAULT
): unknown {
  const message = createAddProvider(authorizedMsaId, schemaIds, expirationBlock);
  const normalized = normalizePayload(message);
  return createEip712Payload(ADD_PROVIDER_DEFINITION, message.type, domain, normalized);
}

/**
 * Returns the EIP-712 browser request for a PaginatedUpsertSignaturePayloadV2 for signing.
 *
 * @param schemaId   uint16 schema identifier
 * @param pageId     uint16 page identifier
 * @param targetHash uint32 page hash
 * @param expiration uint32 expiration block
 * @param payload    HexString or Uint8Array data to upsert
 * @param domain
 */
export function getEip712BrowserRequestPaginatedUpsertSignaturePayloadV2(
  schemaId: number,
  pageId: number,
  targetHash: number,
  expiration: number,
  payload: HexString | Uint8Array,
  domain: EipDomainPayload = EIP712_DOMAIN_DEFAULT
): unknown {
  const message = createPaginatedUpsertSignaturePayloadV2(schemaId, pageId, targetHash, expiration, payload);
  const normalized = normalizePayload(message);
  return createEip712Payload(PAGINATED_UPSERT_SIGNATURE_PAYLOAD_DEFINITION_V2, message.type, domain, normalized);
}

/**
 * Returns the EIP-712 browser request for a PaginatedDeleteSignaturePayloadV2 for signing.
 *
 * @param schemaId   uint16 schema identifier
 * @param pageId     uint16 page identifier
 * @param targetHash uint32 page hash
 * @param expiration uint32 expiration block
 * @param domain
 */
export function getEip712BrowserRequestPaginatedDeleteSignaturePayloadV2(
  schemaId: number,
  pageId: number,
  targetHash: number,
  expiration: number,
  domain: EipDomainPayload = EIP712_DOMAIN_DEFAULT
): unknown {
  const message = createPaginatedDeleteSignaturePayloadV2(schemaId, pageId, targetHash, expiration);
  const normalized = normalizePayload(message);
  return createEip712Payload(PAGINATED_DELETE_SIGNATURE_PAYLOAD_DEFINITION_V2, message.type, domain, normalized);
}

/**
 * Returns the EIP-712 browser request for a ItemizedSignaturePayloadV2 for signing.
 *
 * @param schemaId   uint16 schema identifier
 * @param targetHash uint32 page hash
 * @param expiration uint32 expiration block
 * @param actions    Array of Add/Delete itemized actions
 * @param domain
 */
export function getEip712BrowserRequestItemizedSignaturePayloadV2(
  schemaId: number,
  targetHash: number,
  expiration: number,
  actions: ItemizedAction[],
  domain: EipDomainPayload = EIP712_DOMAIN_DEFAULT
): unknown {
  const message = createItemizedSignaturePayloadV2(schemaId, targetHash, expiration, actions);
  const normalized = normalizePayload(message);
  return createEip712Payload(ITEMIZED_SIGNATURE_PAYLOAD_DEFINITION_V2, message.type, domain, normalized);
}

/**
 * Returns the EIP-712 browser request for a ClaimHandlePayload for signing.
 *
 * @param handle          The handle the user wishes to claim
 * @param expirationBlock Block number after which this payload is invalid
 * @param domain
 */
export function getEip712BrowserRequestClaimHandlePayload(
  handle: string,
  expirationBlock: number,
  domain: EipDomainPayload = EIP712_DOMAIN_DEFAULT
): unknown {
  const message = createClaimHandlePayload(handle, expirationBlock);
  const normalized = normalizePayload(message);
  return createEip712Payload(CLAIM_HANDLE_PAYLOAD_DEFINITION, message.type, domain, normalized);
}

/**
 * Returns the EIP-712 browser request for a PasskeyPublicKey for signing.
 *
 * @param publicKey The passkey’s public key (hex string or raw bytes)
 * @param domain
 */
export function getEip712BrowserRequestPasskeyPublicKey(
  publicKey: HexString | Uint8Array,
  domain: EipDomainPayload = EIP712_DOMAIN_DEFAULT
): unknown {
  const message = createPasskeyPublicKey(publicKey);
  const normalized = normalizePayload(message);
  return createEip712Payload(PASSKEY_PUBLIC_KEY_DEFINITION, message.type, domain, normalized);
}

function createEip712Payload(typeDefinition: SupportedPayloadDefinitions, primaryType: SupportedPayloadTypes, domain: EipDomainPayload, message: NormalizedSupportedPayload): unknown {
  return {
    types: {
      ...EIP712_DOMAIN_DEFINITION,
      ...typeDefinition,
    },
    primaryType,
    domain,
    message,
  };
}

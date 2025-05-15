import { Keypair } from '@polkadot/util-crypto/types';
import {
  AddKeyData,
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
} from './types';
import { assert, isValidHexString, isValidUint16, isValidUint32, isValidUint64 } from './utils';
import { reverseUnifiedAddressToEthereumAddress } from './address';
import { ethers, TypedDataField } from 'ethers';
import { u8aToHex } from '@polkadot/util';
import {
  ADD_KEY_DATA_DEFINITION,
  ADD_PROVIDER_DEFINITION,
  CLAIM_HANDLE_PAYLOAD_DEFINITION,
  ITEMIZED_SIGNATURE_PAYLOAD_DEFINITION,
  PAGINATED_DELETE_SIGNATURE_PAYLOAD_DEFINITION,
  PAGINATED_UPSERT_SIGNATURE_PAYLOAD_DEFINITION,
  PASSKEY_PUBLIC_KEY_DEFINITION,
} from './signature.definitions';

/**
 * Signing EIP-712 compatible signature for payload
 * @param keys
 * @param payload
 * @param chain
 */
export async function signEip712(
  keys: Keypair,
  payload: SupportedPayload,
  chain: ChainType = 'Mainnet-Frequency'
): Promise<EcdsaSignature> {
  // TODO: use correct chainID for different networks
  // using pallet_revive test chain ID for now.
  const chainId = '0x190F1B44';
  // TODO: use correct contract address for different payloads
  const verifyingContract = '0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC';

  // Define the domain separator
  const domainData = {
    name: 'Frequency',
    version: '1',
    chainId,
    verifyingContract,
  };

  const types = getTypesFor(payload.type);
  const normalizedPayload = normalizePayload(payload);
  const wallet = new ethers.Wallet(Buffer.from(keys.secretKey).toString('hex'));
  const signature = await wallet.signTypedData(domainData, types, normalizedPayload);
  return { Ecdsa: signature } as EcdsaSignature;
}

function normalizePayload(payload: SupportedPayload): Record<string, any> {
  const clonedPayload = Object.assign({}, payload);
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
      clonedPayload.newPublicKey = reverseUnifiedAddressToEthereumAddress((payload as AddKeyData).newPublicKey);
      break;

    default:
      throw new Error(`Unsupported payload type: ${JSON.stringify(payload)}`);
  }

  // Remove the type field
  const { type, ...payloadWithoutType } = clonedPayload;

  return payloadWithoutType;
}

function getTypesFor(payloadType: string): Record<string, TypedDataField[]> {
  const PAYLOAD_TYPE_DEFINITIONS: Record<string, Record<string, TypedDataField[]>> = {
    PaginatedUpsertSignaturePayloadV2: PAGINATED_UPSERT_SIGNATURE_PAYLOAD_DEFINITION,
    PaginatedDeleteSignaturePayloadV2: PAGINATED_DELETE_SIGNATURE_PAYLOAD_DEFINITION,
    ItemizedSignaturePayloadV2: ITEMIZED_SIGNATURE_PAYLOAD_DEFINITION,
    PasskeyPublicKey: PASSKEY_PUBLIC_KEY_DEFINITION,
    ClaimHandlePayload: CLAIM_HANDLE_PAYLOAD_DEFINITION,
    AddKeyData: ADD_KEY_DATA_DEFINITION,
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
  const parsedMsaId: bigint = typeof msaId === 'bigint' ? msaId : BigInt(msaId);
  const parsedNewPublicKey: HexString = typeof newPublicKey === 'object' ? u8aToHex(newPublicKey) : newPublicKey;

  assert(isValidUint64(parsedMsaId), 'msaId should be a valid uint32');
  assert(isValidUint32(expirationBlock), 'expiration should be a valid uint32');
  assert(isValidHexString(parsedNewPublicKey), 'newPublicKey should be valid hex');
  return {
    type: 'AddKeyData',
    msaId: parsedMsaId,
    expiration: expirationBlock,
    newPublicKey: parsedNewPublicKey,
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
  const parsedMsaId: bigint = typeof authorizedMsaId === 'bigint' ? authorizedMsaId : BigInt(authorizedMsaId);

  assert(isValidUint64(parsedMsaId), 'targetHash should be a valid uint32');
  assert(isValidUint32(expirationBlock), 'expiration should be a valid uint32');
  schemaIds.forEach((schemaId) => {
    assert(isValidUint16(schemaId), 'schemaId should be a valid uint16');
  });

  return {
    type: 'AddProvider',
    authorizedMsaId: parsedMsaId,
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
  assert(isValidHexString(parsedNewPublicKey), 'publicKey should be valid hex');

  return {
    type: 'PasskeyPublicKey',
    publicKey: parsedNewPublicKey,
  };
}

export function createItemizedAddAction(data: HexString | Uint8Array): AddItemizedAction {
  const parsedData: HexString = typeof data === 'object' ? u8aToHex(data) : data;
  assert(isValidHexString(parsedData), 'itemized data should be valid hex');
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
  assert(isValidHexString(parsedPayload), 'payload should be valid hex');

  return {
    type: 'PaginatedUpsertSignaturePayloadV2',
    schemaId,
    pageId,
    targetHash,
    expiration,
    payload: parsedPayload,
  };
}

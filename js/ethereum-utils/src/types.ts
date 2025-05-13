export type HexString = `0x${string}`;

export interface EcdsaSignature {
  Ecdsa: HexString;
}

export type ChainType = 'Mainnet-Frequency' | 'Paseo-Testnet-Frequency';

export interface PaginatedUpsertSignaturePayloadV2 {
  // type discriminator
  type: 'PaginatedUpsertSignaturePayloadV2';
  // uint16 type schema id
  schemaId: number;
  // uint16 type page id
  pageId: number;
  // uint32 type page hash
  targetHash: number;
  // uint32 type payload expiration block number
  expiration: number;
  // hex encoded data to be stored on Paginated Storage
  payload: HexString;
}

export interface PaginatedDeleteSignaturePayloadV2 {
  // type discriminator
  type: 'PaginatedDeleteSignaturePayloadV2';
  // uint16 type schema id
  schemaId: number;
  // uint16 type page id
  pageId: number;
  // uint32 type page hash
  targetHash: number;
  // uint32 type payload expiration block number
  expiration: number;
}

export interface AddItemizedAction {
  // action item type
  actionType: 'Add';
  // data related to Add item
  data: HexString;
  // uint16 type index related to Delete item
  index: 0;
}

export interface DeleteItemizedAction {
  // action item type
  actionType: 'Delete';
  // data related to Add item
  data: '0x';
  // uint16 type index related to Delete item
  index: number;
}

// Create a union type of the two action types
export type ItemizedAction = AddItemizedAction | DeleteItemizedAction;

export interface ItemizedSignaturePayloadV2 {
  // type discriminator
  type: 'ItemizedSignaturePayloadV2';
  // uint16 type schema id
  schemaId: number;
  // uint32 type page hash
  targetHash: number;
  // uint32 type payload expiration block number
  expiration: number;
  // itemized actions for this payload
  actions: ItemizedAction[];
}

export interface PasskeyPublicKey {
  // type discriminator
  type: 'PasskeyPublicKey';
  // hex encoded public key to be signed
  publicKey: HexString;
}

export interface ClaimHandlePayload {
  // type discriminator
  type: 'ClaimHandlePayload';
  // the handle to be claimed
  handle: string;
  // uint32 type payload expiration block number
  expiration: number;
}

export interface AddKeyData {
  // type discriminator
  type: 'AddKeyData';
  // uint64 type MessageSourceId
  msaId: bigint;
  // uint32 type payload expiration block number
  expiration: number;
  // hex encoded public key to be signed
  newPublicKey: HexString;
}

export interface AddProvider {
  // type discriminator
  type: 'AddProvider';
  // uint64 type MessageSourceId
  authorizedMsaId: bigint;
  // uint16[] type schema ids
  schemaIds: number[];
  // uint32 type payload expiration block number
  expiration: number;
}

export type SupportedPayload =
  | PaginatedUpsertSignaturePayloadV2
  | PaginatedDeleteSignaturePayloadV2
  | ItemizedSignaturePayloadV2
  | PasskeyPublicKey
  | ClaimHandlePayload
  | AddKeyData
  | AddProvider;

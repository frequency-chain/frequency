export type HexString = `0x${string}`;

export interface EcdsaSignature {
  Ecdsa: HexString;
}

export interface Address20MultiAddress {
  Address20: number[];
}

export type ChainType = 'Mainnet-Frequency' | 'Paseo-Testnet-Frequency' | 'Dev';

export interface AddressWrapper {
  // 20 byte ethereum address in hex
  ethereumAddress: HexString;
  // 32 byte unified address in hex
  unifiedAddress: HexString;
  // SS58 unified address
  unifiedAddressSS58: string;
}

export interface EthereumKeyPair {
  // private key of ethereum key
  privateKey: HexString;
  // 33 bytes public key
  publicKey: HexString;
  // different address representations
  address: AddressWrapper;
  // 12 word mnemonic
  mnemonic: string;
}

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
  msaId: string;
  // uint32 type payload expiration block number
  expiration: number;
  // hex encoded public key to be signed
  newPublicKey: HexString;
}

export interface AuthorizedKeyData {
  // type discriminator
  type: 'AuthorizedKeyData';
  // uint64 type MessageSourceId
  msaId: string;
  // uint32 type payload expiration block number
  expiration: number;
  // hex encoded public key to be signed
  authorizedPublicKey: HexString;
}

export interface AddProvider {
  // type discriminator
  type: 'AddProvider';
  // uint64 type MessageSourceId
  authorizedMsaId: string;
  // uint16[] type schema ids
  schemaIds: number[];
  // uint32 type payload expiration block number
  expiration: number;
}

export interface RecoveryCommitmentPayload {
  // type discriminator
  type: 'RecoveryCommitmentPayload';
  // HexString type recovery commitment
  recoveryCommitment: HexString;
  // uint32 type payload expiration block number
  expiration: number;
}

export interface SiwfSignedRequestPayload {
  // type discriminator
  type: 'SiwfSignedRequestPayload';
  // callback url
  callback: string;
  // uint16[] type schema ids
  permissions: number[];
  // Only used for custom integration situations.
  userIdentifierAdminUrl?: string;
}

export interface SiwfLoginRequestPayload {
  // type discriminator
  type: 'SiwfLoginRequestPayload';
  // message url
  message: string;
}

export type SupportedPayload =
  | PaginatedUpsertSignaturePayloadV2
  | PaginatedDeleteSignaturePayloadV2
  | ItemizedSignaturePayloadV2
  | PasskeyPublicKey
  | ClaimHandlePayload
  | AddKeyData
  | AuthorizedKeyData
  | AddProvider
  | RecoveryCommitmentPayload
  | SiwfSignedRequestPayload
  | SiwfLoginRequestPayload;

export type NormalizedSupportedPayload = Omit<SupportedPayload, 'type'>;

export interface EipDomainPayload {
  name: string;
  version: string;
  chainId: HexString;
  verifyingContract: HexString;
}

export type SupportedPayloadTypes = SupportedPayload['type'];

export type SignatureType = 'EIP-712' | 'EIP-191';

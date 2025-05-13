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
} from './types';
import { assert, isValidHexString, isValidUint16, isValidUint32, isValidUint64 } from './utils';
import { reverseUnifiedAddressToEthereumAddress } from './address';
import { ethers, TypedDataField } from 'ethers';

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
  const normalizedPayload = checkAndNormalizePayload(payload);
  const wallet = new ethers.Wallet(Buffer.from(keys.secretKey).toString('hex'));
  const signature = await wallet.signTypedData(domainData, types, normalizedPayload);
  return { Ecdsa: signature } as EcdsaSignature;
}

function checkAndNormalizePayload(payload: SupportedPayload): Record<string, any> {
  const clonedPayload = Object.assign({}, payload);
  switch (clonedPayload.type) {
    case 'PaginatedUpsertSignaturePayloadV2':
      assert(isValidUint16(clonedPayload.schemaId), 'schemaId should be a valid uint16');
      assert(isValidUint16(clonedPayload.pageId), 'pageId should be a valid uint16');
      assert(isValidUint32(clonedPayload.targetHash), 'targetHash should be a valid uint32');
      assert(isValidUint32(clonedPayload.expiration), 'expiration should be a valid uint32');
      assert(isValidHexString(clonedPayload.payload), 'payload should be valid hex');
      break;

    case 'PaginatedDeleteSignaturePayloadV2':
      assert(isValidUint16(clonedPayload.schemaId), 'schemaId should be a valid uint16');
      assert(isValidUint16(clonedPayload.pageId), 'pageId should be a valid uint16');
      assert(isValidUint32(clonedPayload.targetHash), 'targetHash should be a valid uint32');
      assert(isValidUint32(clonedPayload.expiration), 'expiration should be a valid uint32');
      break;

    case 'ItemizedSignaturePayloadV2':
      assert(isValidUint16(clonedPayload.schemaId), 'schemaId should be a valid uint16');
      assert(isValidUint32(clonedPayload.targetHash), 'targetHash should be a valid uint32');
      assert(isValidUint32(clonedPayload.expiration), 'expiration should be a valid uint32');
      clonedPayload.actions.forEach((item) => {
        switch (item.actionType) {
          case 'Add':
            assert(isValidHexString(item.data), 'itemized data should be valid hex');
            assert(item.index === 0);
            break;
          case 'Delete':
            assert(isValidUint16(item.index), 'itemized index should be a valid uint16');
            assert(item.data === '0x');
            break;
        }
      });
      break;

    case 'PasskeyPublicKey':
      assert(isValidHexString(clonedPayload.publicKey), 'publicKey should be valid hex');
      break;

    case 'ClaimHandlePayload':
      assert(isValidUint32(clonedPayload.expiration), 'expiration should be a valid uint32');
      assert(clonedPayload.handle.length > 0, 'handle should be a valid string');
      break;

    case 'AddKeyData':
      assert(isValidUint64(clonedPayload.msaId), 'msaId should be a valid uint32');
      assert(isValidUint32(clonedPayload.expiration), 'expiration should be a valid uint32');
      assert(isValidHexString(clonedPayload.newPublicKey), 'newPublicKey should be valid hex');
      // convert to 20 bytes ethereum address for signature
      clonedPayload.newPublicKey = reverseUnifiedAddressToEthereumAddress((payload as AddKeyData).newPublicKey);

      break;

    case 'AddProvider':
      assert(isValidUint64(clonedPayload.authorizedMsaId), 'targetHash should be a valid uint32');
      assert(isValidUint32(clonedPayload.expiration), 'expiration should be a valid uint32');
      clonedPayload.schemaIds.forEach((schemaId) => {
        assert(isValidUint16(schemaId), 'schemaId should be a valid uint16');
      });
      break;

    default:
      throw new Error(`Unsupported payload type: ${JSON.stringify(payload)}`);
  }

  // Remove the type field
  const { type, ...payloadWithoutType } = clonedPayload;

  return payloadWithoutType;
}

function getTypesFor(payloadType: string): Record<string, TypedDataField[]> {
  switch (payloadType) {
    case 'PaginatedUpsertSignaturePayloadV2':
      return PAGINATED_UPSERT_SIGNATURE_PAYLOAD_DEFINITION;

    case 'PaginatedDeleteSignaturePayloadV2':
      return PAGINATED_DELETE_SIGNATURE_PAYLOAD_DEFINITION;

    case 'ItemizedSignaturePayloadV2':
      return ITEMIZED_SIGNATURE_PAYLOAD_DEFINITION;

    case 'PasskeyPublicKey':
      return PASSKEY_PUBLIC_KEY_DEFINITION;

    case 'ClaimHandlePayload':
      return CLAIM_HANDLE_PAYLOAD_DEFINITION;

    case 'AddKeyData':
      return ADD_KEY_DATA_DEFINITION;

    case 'AddProvider':
      return ADD_PROVIDER_DEFINITION;
  }
  throw new Error(`Unsupported payload type: ${payloadType}`);
}

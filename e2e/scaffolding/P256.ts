import { SubmittableExtrinsic } from '@polkadot/api/types';
import { base64UrlToUint8Array } from './helpers';
import { secp256r1 } from '@noble/curves/p256';
import { ISubmittableResult } from '@polkadot/types/types';
import { u8aWrapBytes } from '@polkadot/util';
import { ExtrinsicHelper } from './extrinsicHelpers';
import { sha256 } from '@noble/hashes/sha256';
import {MultiSignature} from "@polkadot/types/interfaces";

export function createPassKeyAndSignAccount(accountPKey: Uint8Array) {
  const passKeyPrivateKey = secp256r1.utils.randomPrivateKey();
  const passKeyPublicKey = secp256r1.getPublicKey(passKeyPrivateKey, true);
  const passkeySignature = secp256r1.sign(u8aWrapBytes(accountPKey), passKeyPrivateKey).toDERRawBytes();
  return { passKeyPrivateKey, passKeyPublicKey, passkeySignature };
}

export async function createPassKeyCall(
  accountPKey: Uint8Array,
  nonce: number,
  accountSignature: MultiSignature,
  call: SubmittableExtrinsic<'rxjs', ISubmittableResult>
) {
  const ext_call_type = ExtrinsicHelper.api.registry.createType('Call', call);
  const passkeyCall = {
    accountId: accountPKey,
    accountNonce: nonce,
    accountOwnershipProof: accountSignature,
    call: ext_call_type,
  };

  return passkeyCall;
}

export async function createPasskeyPayload(
  passKeyPrivateKey: Uint8Array,
  passKeyPublicKey: Uint8Array,
  passkeyCallPayload: any = {},
  set_invalid_passkey_data: boolean = false
) {
  const authenticatorDataRaw = 'WJ8JTNbivTWn-433ubs148A7EgWowi4SAcYBjLWfo1EdAAAAAA';
  const replacedClientDataRaw =
    'eyJ0eXBlIjoid2ViYXV0aG4uZ2V0IiwiY2hhbGxlbmdlIjoiI3JwbGMjIiwib3JpZ2luIjoiaHR0cHM6Ly9wYXNza2V5LmFtcGxpY2EuaW86ODA4MCIsImNyb3NzT3JpZ2luIjpmYWxzZSwiYWxnIjoiSFMyNTYifQ';
  const challengeReplacer = '#rplc#';
  let clientData = base64UrlToUint8Array(replacedClientDataRaw);
  let authenticatorData = base64UrlToUint8Array(authenticatorDataRaw);

  if (set_invalid_passkey_data) {
    authenticatorData = new Uint8Array(0);
    clientData = new Uint8Array(0);
  }
  const passkeyCallType = ExtrinsicHelper.api.createType('PalletPasskeyPasskeyCall', passkeyCallPayload);

  // Challenge is sha256(passkeyCallType)
  const calculatedChallenge = sha256(passkeyCallType.toU8a());
  const calculatedChallengeBase64url = Buffer.from(calculatedChallenge).toString('base64url');
  // inject challenge inside clientData
  const clientDataJSON = Buffer.from(clientData)
    .toString('utf-8')
    .replace(challengeReplacer, calculatedChallengeBase64url);
  // prepare signing payload which is [authenticator || sha256(client_data_json)]
  const passkeySha256 = sha256(new Uint8Array([...authenticatorData, ...sha256(Buffer.from(clientDataJSON))]));
  const passKeySignature = secp256r1.sign(passkeySha256, passKeyPrivateKey).toDERRawBytes();
  const passkeyPayload = {
    passkeyPublicKey: Array.from(passKeyPublicKey),
    verifiablePasskeySignature: {
      signature: Array.from(passKeySignature),
      authenticatorData: Array.from(authenticatorData),
      clientDataJson: Array.from(Buffer.from(clientDataJSON)),
    },
    passkeyCall: passkeyCallType,
  };
  const payload = ExtrinsicHelper.api.createType('PalletPasskeyPasskeyPayload', passkeyPayload);

  return payload;
}

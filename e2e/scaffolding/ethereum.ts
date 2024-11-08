import {KeyringPair} from "@polkadot/keyring/types";
import {encodeAddress, ethereumEncode} from "@polkadot/util-crypto";
import {hexToU8a, u8aToHex} from "@polkadot/util";
import {MultiAddress} from "@polkadot/types/interfaces";
import type {Signer} from "@polkadot/types/types";
import {SignerResult} from "@polkadot/types/types";
import { secp256k1 } from '@noble/curves/secp256k1';
import {Keyring} from "@polkadot/api";
import {Keypair} from "@polkadot/util-crypto/types";
import {keccak256} from "@polkadot/wasm-crypto";
import assert from "assert";

export function getUnifiedAddress(pair: KeyringPair) : string {
  if (['ecdsa','ethereum'].includes(pair.type)) {
    const etheAddressHex = ethereumEncode(pair.publicKey);
    return getConvertedEthereumAccount(etheAddressHex)
  }
  return pair.address
}

export function getEthereumStyleSigner(ethereumPair: KeyringPair) : Signer {
  return {
    signRaw: async (payload): Promise<SignerResult> => {
      console.log(`raw_payload: ${payload.data}`);
      const sig = ethereumPair.sign(wrapCustomEthereumTags(payload.data));
      const prefixedSignature = new Uint8Array(sig.length + 1);
      prefixedSignature[0]=2;
      prefixedSignature.set(sig, 1);
      const hex = u8aToHex(prefixedSignature);
      return {
        signature: hex,
      } as SignerResult;
    },
  }
}

export function getEthereumStyleSignerTest(expectedPayloadHex: string, injectedSignatureHex: string) : Signer {
  return {
    signRaw: async (payload): Promise<SignerResult> => {
      console.log(`raw_payload: ${payload.data}`);
      assert.equal(payload.data, expectedPayloadHex);
      const sig = hexToU8a(injectedSignatureHex);
      const prefixedSignature = new Uint8Array(sig.length + 1);
      prefixedSignature[0]=2;
      prefixedSignature.set(sig, 1);
      const hex = u8aToHex(prefixedSignature);
      return {
        signature: hex,
      } as SignerResult;
    },
  }
}

function wrapCustomEthereumTags(hexPayload: string) : Uint8Array {
  // wrapping in frequency tags to show this is a Frequency related payload
  const frequencyWrapped = `<Frequency>${hexPayload.toLowerCase()}</Frequency>`
  // prefixing with the EIP-191 for personal_sign messages (this gets wrapped automatically in metamask)
  const wrapped = `\x19Ethereum Signed Message:\n${frequencyWrapped.length}${frequencyWrapped}`
  console.log(`wrapped ${wrapped}`);
  const buffer =  Buffer.from(wrapped, "utf-8");
  return new Uint8Array(buffer.buffer, buffer.byteOffset, buffer.length);
}

export function getAccountId20MultiAddress(pair: KeyringPair): MultiAddress {
  const etheAddress = ethereumEncode(pair.publicKey);
  let ethAddress20 = Array.from(hexToU8a(etheAddress));
  return {
      Address20: ethAddress20
    } as MultiAddress;
}

export function getConvertedEthereumPublicKey(pair: KeyringPair): Uint8Array {
  const publicKeyBytes = hexToU8a(ethereumEncode(pair.publicKey));
  const result = new Uint8Array(32);
  result.fill(0, 0, 12);
  result.set(publicKeyBytes, 12);
  return result;
}

function getConvertedEthereumAccount(
  accountId20Hex: string
) : string {
  const addressBytes = hexToU8a(accountId20Hex);
  const result = new Uint8Array(32);
  result.fill(0, 0, 12);
  result.set(addressBytes, 12);
  return encodeAddress(result);
}

/**
 *
 * @param secretKey of secp256k1 keypair exported from any wallet (should be 32 bytes)
 */
export function getKeyringPairFromSecp256k1PrivateKey(secretKey: Uint8Array): KeyringPair {
  const publicKey = secp256k1.getPublicKey(secretKey, true);
  const keypair: Keypair = {
    secretKey,
    publicKey
  };
  const keyring = new Keyring({ type: 'ethereum' });
  return keyring.addFromPair(keypair, undefined, 'ethereum' )
}

import {KeyringPair} from "@polkadot/keyring/types";
import {encodeAddress, ethereumEncode} from "@polkadot/util-crypto";
import {hexToU8a, u8aToHex} from "@polkadot/util";
import {MultiAddress} from "@polkadot/types/interfaces";
import type {Signer} from "@polkadot/types/types";
import {SignerResult} from "@polkadot/types/types";

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
      const sig = ethereumPair.sign(payload.data);
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

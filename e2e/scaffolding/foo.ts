import { getUnifiedAddress, getUnifiedPublicKey } from "./ethereum";
import { KeyringPair } from '@polkadot/keyring/types';
import { Keyring } from '@polkadot/api';
import { mnemonicGenerate } from '@polkadot/util-crypto';

const ethKeys = new Keyring({ type: 'ethereum' }).createFromUri(mnemonicGenerate());
const ethAddress = getUnifiedAddress(ethKeys);
const ethPublicKey = getUnifiedPublicKey(ethKeys);

const ethAddrss2 = getUnifiedAddress({ type: 'ethereum', publicKey: ethPublicKey} as unknown as KeyringPair);

console.log('Results: ', { ethPublicKey, ethAddress, ethAddrss2 });

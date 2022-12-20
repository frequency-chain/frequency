import { ApiRx, Keyring } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import { Compact, u128 } from "@polkadot/types";
import { DispatchError, Event } from "@polkadot/types/interfaces";
import { AnyNumber, Codec, ISubmittableResult } from "@polkadot/types/types";
import { u8aToHex, u8aWrapBytes } from "@polkadot/util";
import { mnemonicGenerate } from '@polkadot/util-crypto';
import { pipe, map, tap, of } from "rxjs";
import { createKeys } from "./apiConnection";
import { getAccountInfo, transferFunds } from "./extrinsicHelpers";

export const INITIAL_FUNDING = 100000000n;

export enum DevAccounts {
    Alice = "//Alice",
    Bob = "//Bob",
    Charlie = "//Charlie",
    Dave = "//Dave",
    Eve = "//Eve",
    Ferdie = "//Ferdie",
}

export class EventError extends Error {
    name: string = '';
    message: string = '';
    stack?: string = '';
    section?: string = '';
    rawError: DispatchError;

    constructor(source: DispatchError) {
        super();

        if (source) {
            if (source.isModule) {
                const decoded = source.registry.findMetaError(source.asModule);
                this.name = decoded.name;
                this.message = decoded.docs.join(' ');
                this.section = decoded.section;
            } else {
                this.name = source.type;
                this.message = source.type;
                this.section = '';
            }
        }
        this.rawError = source;
    }

    public toString() {
        return `${this.section}.${this.name}: ${this.message}`;
    }
}

export type EventMap = { [key: string]: Event }
export type Sr25519Signature = { Sr25519: `0x${string}` }

interface BeginEndBalances {
    beginBalance: bigint;
    endBalance?: bigint;
}
let balancesMap = new Map<string, Map<string, BeginEndBalances>>;

export function parseResult() {
    return pipe(
        tap((result: ISubmittableResult) => {
            if (result.dispatchError) {
                let err: EventError;

                switch (result.dispatchError.type) {
                    case 'Module': {
                        err = new EventError(result.dispatchError);
                        break;
                    }

                    default: {
                        err = new EventError(result.dispatchError);
                        break;
                    }
                }

                console.log(err.toString());
                throw err;
            }
        }),
        map((result: ISubmittableResult) => result.events.reduce((acc, { event }) => { acc[eventKey(event)] = event.data; return acc }, {})),
        // tap((events) => console.log(events)),
    );
}

export function eventKey(event: Event): string {
    return `${event.section}.${event.method}`;
}

export function signPayloadSr25519(keys: KeyringPair, data: Codec): Sr25519Signature {
    return { Sr25519: u8aToHex(keys.sign(u8aWrapBytes(data.toU8a()))) }
}

export function createAccount(name: string = 'first pair') {
    const mnemonic = mnemonicGenerate();
    // create & add the pair to the keyring with the type and some additional
    // metadata specified
    const keyring = new Keyring({ type: 'sr25519' });
    const keypair = keyring.addFromUri(mnemonic, { name }, 'sr25519');

    return keypair;
}

export async function createAndFundAccount({ api, amount, source, name, context }:
    { api: ApiRx, amount: bigint, source: DevAccounts, name?: string, context?: string }) {
    const keypair = createAccount(name);

    // Get keypair for pre-funded dev account
    const devKeys = createKeys(source);

    // Transfer funds from pre-funded dev account to new account
    await transferFunds(api, devKeys, keypair, amount);
    let map = balancesMap.get(context || 'undefined');
    if (!map) {
        map = new Map<string, BeginEndBalances>();
    }

    map.set(keypair.address, { beginBalance: amount });
    balancesMap.set(context || 'undefined', map);

    return { newAccount: keypair, devAccount: devKeys };
}

export async function showTotalCost(api: ApiRx, context?: string) {
    let cum = 0n;
    const titleStr = context ? `[${context}]: ` : '';
    const map = balancesMap.get(context || 'undefined');
    if (!map) {
        return;
    }
    console.log(`${titleStr}Operation costs were: `);
    for (const [key, value] of map) {
        const acct = await getAccountInfo(api, key);
        value.endBalance = acct.data.free.toBigInt();
        const cost = value.beginBalance - value.endBalance;
        cum += cost;
        console.log(`${titleStr}${key}: ${cost}`);
    }
    console.log(`${titleStr}Total cost of all operations was (excluding standard dev accounts): ${cum}`);
}

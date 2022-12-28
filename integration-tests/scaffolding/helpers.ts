import { Keyring } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import { DispatchError, Event } from "@polkadot/types/interfaces";
import { Codec, ISubmittableResult } from "@polkadot/types/types";
import { u8aToHex, u8aWrapBytes } from "@polkadot/util";
import { mnemonicGenerate } from '@polkadot/util-crypto';
import { pipe, map, tap, firstValueFrom } from "rxjs";
import { env } from "./env";
import { Extrinsic, ExtrinsicHelper } from "./extrinsicHelpers";
import { EXISTENTIAL_DEPOSIT } from "./rootHooks";

export const INITIAL_FUNDING = 100000000n;

export interface DevAccount {
    uri: string,
    keys: KeyringPair,
}

export let devAccounts: DevAccount[] = [];


export class EventError extends Error {
    name: string = '';
    message: string = '';
    stack?: string = '';
    section?: string = '';
    rawError: DispatchError;

    constructor(source: DispatchError) {
        super();

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
        this.rawError = source;
    }

    public toString() {
        return `${this.section}.${this.name}: ${this.message}`;
    }
}

export type EventMap = { [key: string]: Event }
export type Sr25519Signature = { Sr25519: `0x${string}` }

export function parseResult() {
    return pipe(
        tap((result: ISubmittableResult) => {
            if (result.dispatchError) {
                let err = new EventError(result.dispatchError);
                log(err.toString());
                throw err;
            }
        }),
        map((result: ISubmittableResult) => result.events.reduce((acc, { event }) => { acc[eventKey(event)] = event; return acc }, {})),
        tap((events) => log(events)),
    );
}

export function eventKey(event: Event): string {
    return `${event.section}.${event.method}`;
}

export function signPayloadSr25519(keys: KeyringPair, data: Codec): Sr25519Signature {
    return { Sr25519: u8aToHex(keys.sign(u8aWrapBytes(data.toU8a()))) }
}

export function createKeys(name: string = 'first pair'): KeyringPair {
    const mnemonic = mnemonicGenerate();
    // create & add the pair to the keyring with the type and some additional
    // metadata specified
    const keyring = new Keyring({ type: 'sr25519' });
    const keypair = keyring.addFromUri(mnemonic, { name }, 'sr25519');

    return keypair;
}

export async function fundKeypair(source: KeyringPair, dest: KeyringPair, amount: bigint): Promise<void> {
    await ExtrinsicHelper.transferFunds(source, dest, amount).signAndSend();
}

export async function createAndFundKeypair(amount = EXISTENTIAL_DEPOSIT, keyName?: string): Promise<KeyringPair> {
    const keypair = createKeys(keyName);

    // Transfer funds from source (usually pre-funded dev account) to new account
    await fundKeypair(devAccounts[0].keys, keypair, amount);

    return keypair;
}

export function log(...args: any[]) {
    if (env.verbose) {
        console.log(...args);
    }
}

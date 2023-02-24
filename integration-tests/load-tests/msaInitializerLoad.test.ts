import "@frequency-chain/api-augment";
import assert from "assert";
import { createKeys, signPayloadSr25519, getBlockNumber, generateAddKeyPayload, createAndFundKeypair } from "../scaffolding/helpers";
import { KeyringPair } from "@polkadot/keyring/types";
import { AddKeyData, ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { EXISTENTIAL_DEPOSIT } from "../scaffolding/rootHooks";
import { firstValueFrom } from "rxjs";
import { BlockHash, CreatedBlock } from "@polkadot/types/interfaces";
import { u64, Option } from "@polkadot/types";

interface GeneratedMsa {
    id: u64;
    nonce: number;
    controlKey: KeyringPair;
}

describe("MSA Initializer Load Tests", function () {

    beforeEach(async function () {
        await createBlock();
    });

    it("should successfully create 49_998 signatures within 100 blocks", async function () {

        // Make 250 MSAs
        let msaKeys: GeneratedMsa[] = [];
        for (let i = 0; i < 2; i++) { // Should be 250
             msaKeys.push(await generateMsa());
        }

        await createBlock();

        const blockNumber = await getBlockNumber();
        console.log("Starting Block Number:", blockNumber);

        const signatureCallTests: Array<KeyringPair> = [];

        // Make 49,998 signatures
        for (let i = 0; i < 2; i++) { // Should be 24_998
            const ithMsaKey = i % msaKeys.length;
            let { controlKey, nonce, id } = msaKeys[ithMsaKey];

            if (i > 0 && i % 330 === 0) {
                await createBlock();
                console.log(`Forming block...`);
            }

            // This call adds 2 signatures
            signatureCallTests.push(addSigs(id, controlKey, nonce));
            console.count("addSigs called (2 signatures)");
            msaKeys[ithMsaKey].nonce++;
        }
        console.log(`Forming block...`);
        await createBlock();
        console.log(`Forming block...`);
        await createBlock();
        const blockNumberEnd = await getBlockNumber();
        console.log("Ending Block Number:", blockNumberEnd);

        // Check to make sure all of them got created
        for (let key of signatureCallTests) {
            assert((await getMsaFromKey(key)).isSome, "A key failed to be associated with an MSA");
        }

    }).timeout(500_000)
})

// Generate an MSA and give it 100 UNIT
async function generateMsa(): Promise<GeneratedMsa> {
    console.count("generateMsa");
    const controlKeyPromise = createAndFundKeypair(100n * 10n * EXISTENTIAL_DEPOSIT);
    await createBlock(100);
    const controlKey = await controlKeyPromise;
    const id = await createMsa(controlKey);
    await createBlock(100);
    const nonce = await getNonce(controlKey);
    return {
        controlKey,
        id,
        nonce: nonce + 1,
    }
}

async function createBlock(wait: number = 300) {
    // Wait ms before creating the block to give the chain time to process the transaction pool
    await new Promise(r => setTimeout(r, wait));
    return firstValueFrom(ExtrinsicHelper.api.rpc.engine.createBlock(true, true));
}

function getMsaFromKey(keys: KeyringPair): Promise<Option<u64>> {
    return firstValueFrom(ExtrinsicHelper.api.query.msa.publicKeyToMsaId(keys.address));
}

function getNonce(keys: KeyringPair): Promise<number> {
    return firstValueFrom(ExtrinsicHelper.api.call.accountNonceApi.accountNonce(keys.address));
}

async function createMsa(keys: KeyringPair): Promise<u64> {
    const f = ExtrinsicHelper.createMsa(keys);
    f.signAndSend();
    let block:CreatedBlock = await firstValueFrom(ExtrinsicHelper.api.rpc.engine.createBlock(true, true));

    const events = await getBlockEvents(block.blockHash);
    let msaId;
    events.forEach((record) => {
        let { event } = record;
        if (event.section === "msa" && event.method === "MsaCreated") { msaId = event.data[0] }
    })
    return msaId;
}

function addSigs(msaId: u64, keys: KeyringPair, nonce: number): KeyringPair {
    let newKeys = createKeys(`${nonce} nonce control key`);

    let defaultPayload: AddKeyData = {};
    defaultPayload.msaId = msaId;
    defaultPayload.newPublicKey = newKeys.publicKey;
    generateAddKeyPayload(defaultPayload, 100).then((payload) => {
        let addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", payload);

        let ownerSig = signPayloadSr25519(keys, addKeyData);
        let newSig = signPayloadSr25519(newKeys, addKeyData);
        const addPublicKeyOp = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, newSig, payload);

        return addPublicKeyOp.signAndSend(nonce);
    });

    return newKeys;
}

function getBlockEvents(blockHash: BlockHash) {
    const blockEvents = ExtrinsicHelper.api.query.system.events.at(blockHash)
    return firstValueFrom(blockEvents);
}

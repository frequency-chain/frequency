import "@frequency-chain/api-augment";
import assert from "assert";
import { createKeys, signPayloadSr25519, getBlockNumber, generateAddKeyPayload, fundKeypair, devAccounts, createAndFundKeypairManual } from "../scaffolding/helpers";
import { KeyringPair } from "@polkadot/keyring/types";
import { AddKeyData, ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { EXISTENTIAL_DEPOSIT } from "../scaffolding/rootHooks";
import { firstValueFrom } from "rxjs";
import { BlockHash, CreatedBlock, Index } from "@polkadot/types/interfaces";
import { BN, u8aToHex } from "@polkadot/util";
import { U64, u64 } from "@polkadot/types";

interface GeneratedMsa {
    id: u64;
    nonce: number;
    controlKey: KeyringPair;
}

describe("MSA Initializer Load Tests", function () {

    beforeEach(async function () {
        await createBlock();
    });

    it("should successfully create 49_999 signatures within a block", async function () {

        // Our bucket size is 100 blocks.
        // To reach our 50k max,
        // we should be limiting 500 signatures per block. msa_addPublicKeyToMsa adds 2 signatures per call. Therefore, we need to create a new block
        // when the counter hits 250 (500 / 2).

        let msaKeys: GeneratedMsa[] = [];
        for (let i = 0; i < 2_500; i++) {
             msaKeys.push(await generateMsa());
        }

        await createBlock();

        // Make sure we are at [block number] % 100 so we get all these in the same bucket
        const blockNumber = await getBlockNumber();
        if (blockNumber % 100 > 0) {
            console.log("Getting to a mod(100) block number. Starting Block Number:", blockNumber);
            for (let i = (blockNumber % 100); i < 100; i++) {
                await createBlock();
            }
            console.log("DONE! Ending Block Number:", await getBlockNumber());
        }

        for (let i = 0; i < 30_000; i++) {
            const ithMsaKey = i % msaKeys.length;
            let { controlKey, nonce, id } = msaKeys[ithMsaKey];
            console.log(`${i} pair of sigs`);

            if (i > 0 && i % 330 === 0) {
                await new Promise(r => setTimeout(r, 300));
                await createBlock();
                console.log(`${i} block`);
            }
            // This call adds 2 signatures
            addSigs(id, controlKey, nonce);
            msaKeys[ithMsaKey].nonce++;
        }
        // Create another few blocks at the end
        await new Promise(r => setTimeout(r, 300));
        await createBlock();
        await new Promise(r => setTimeout(r, 300));
        await createBlock();
        await new Promise(r => setTimeout(r, 300));
        await createBlock();
    }).timeout(500_000)
})

async function generateMsa(): Promise<GeneratedMsa> {
    console.count("generateMsa");
    const controlKey = await createAndFundKeypairManual(50n * EXISTENTIAL_DEPOSIT);
    await createBlock();
    const id = await createMsa(controlKey);
    await createBlock();
    const nonce = await getNonce(controlKey.address);
    return {
        controlKey,
        id,
        nonce: nonce + 1,
    }
}

async function createBlock() {
    return firstValueFrom(ExtrinsicHelper.api.rpc.engine.createBlock(true, true));
}

async function getNonce(keys): Promise<number> {
    return firstValueFrom(ExtrinsicHelper.api.call.accountNonceApi.accountNonce(keys.address));
}

async function createMsa(keys): Promise<u64> {
    const f = ExtrinsicHelper.createMsa(keys);
    await f.signAndSendManual();
    let block:CreatedBlock = await firstValueFrom(ExtrinsicHelper.api.rpc.engine.createBlock(true, true));

    const events = await getBlockEvents(block.blockHash);
    let msaId;
    events.forEach((record) => {
        let { event } = record;
        if (event.section === "msa" && event.method === "MsaCreated") { msaId = event.data[0] }
    })
    return msaId;
}

async function addSigs(msaId, keys, nonce) {
    let newKeys = createKeys(`${nonce} nonce control key`);

    let defaultPayload: AddKeyData = {};
    defaultPayload.msaId = msaId;
    defaultPayload.newPublicKey = newKeys.publicKey;
    let payload = await generateAddKeyPayload(defaultPayload);
    let addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", payload);

    let ownerSig = signPayloadSr25519(keys, addKeyData);
    let newSig = signPayloadSr25519(newKeys, addKeyData);
    const addPublicKeyOp = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, newSig, payload);

    addPublicKeyOp.signAndSendManual(nonce);
}

async function getBlockEvents(blockHash: BlockHash) {
    const blockEvents = await ExtrinsicHelper.api.query.system.events.at(blockHash)
    return firstValueFrom(blockEvents);
}

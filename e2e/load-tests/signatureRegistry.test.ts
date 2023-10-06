import "@frequency-chain/api-augment";
import assert from "assert";
import { createKeys, signPayloadSr25519, getBlockNumber, generateAddKeyPayload, createAndFundKeypair, getNonce, getExistentialDeposit } from "../scaffolding/helpers";
import { KeyringPair } from "@polkadot/keyring/types";
import { AddKeyData, ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { firstValueFrom } from "rxjs";
import { u64, Option } from "@polkadot/types";
import { getFundingSource } from "../scaffolding/funding";

interface GeneratedMsa {
    id: u64;
    nonce: number;
    controlKey: KeyringPair;
}

describe("MSA Initializer Load Tests", function () {
    const fundingSource = getFundingSource("load-signature-registry");

    beforeEach(async function () {
        await createBlock();
    });

    it("should successfully create 50k signatures within 100 blocks", async function () {

        const msaKeys: GeneratedMsa[] = await generateMsas(2300);

        let blockNumber = await getBlockNumber();
        console.log("Starting Block Number:", blockNumber);

        let signatureCallTests: Array<KeyringPair> = [];

        // Make 50k signatures via 50k/2 loops
        const loopCount = 25_000;
        for (let i = 0; i < loopCount; i++) {
            const ithMsaKey = i % msaKeys.length;
            let { controlKey, nonce, id } = msaKeys[ithMsaKey];

            // This call adds 2 signatures
            signatureCallTests.push(await addSigs(id, controlKey, blockNumber, nonce));
            console.count("addSigs called (2 signatures)");
            msaKeys[ithMsaKey].nonce++;

            // Create block and check...
            if (i > 0 && i % 330 === 0) {
                await createBlock(200);
                blockNumber++;
                console.log(`Forming block...`);
                await checkKeys(i - 330, [...signatureCallTests]);
                signatureCallTests = [];
            }

        }
        console.log(`Forming block...`);
        await createBlock();

        // Final Check
        await checkKeys(loopCount - (loopCount % 330), signatureCallTests);

        const blockNumberEnd = await getBlockNumber();
        console.log("Ending Block Number:", blockNumberEnd);

    }).timeout(5_000_000)
})

// Checks the keys to make sure they were associated with an MSA
// Also will form a block in case something was still inside the transaction queue
async function checkKeys(startingNumber: number, keysToTest: KeyringPair[]) {
    console.log(`Testing MSA Key connections for ${keysToTest.length} keys...`);
    for (let i = 0; i < keysToTest.length; i++) {
        const key = keysToTest[i];
        let msaOption = await getMsaFromKey(key);
        if (!msaOption.isSome) {
            console.log(`The ${startingNumber + i} key (${key.address}) failed to be associated with an MSA. Trying another block...`);
            await createBlock();
            msaOption = await getMsaFromKey(key);

        }
        assert(msaOption.isSome, `The ${startingNumber + i} key (${key.address}) failed to be associated with an MSA.`);
    }
}

// Generate MSAs and give it 100 UNIT
async function generateMsas(count: number = 1): Promise<GeneratedMsa[]> {
    // Make sure we are not on an edge
    const createBlockEvery = count === 300 ? 290 : 300;
    const fundingSource = getFundingSource("load-signature-registry");

    // Create and fund the control keys
    let controlKeyPromises: Array<Promise<KeyringPair>> = [];
    let devAccountNonce = await getNonce(fundingSource);
    const ed = await getExistentialDeposit();
    for (let i = 0; i < count; i++) {
        controlKeyPromises.push(createAndFundKeypair(fundingSource, 100n * 10n * ed, undefined, devAccountNonce++));
        if (i > 0 && i % createBlockEvery === 0) await createBlock(100);
    }
    await createBlock();
    const controlKeys = await Promise.all(controlKeyPromises);

    // Create the msas
    let msaPromises: Array<Promise<GeneratedMsa>> = [];
    for (let i = 0; i < count; i++) {
        msaPromises.push(createMsa(controlKeys[i]).then((id) => ({
                controlKey: controlKeys[i],
                id,
                nonce: 1,
            }))
        );
        if (i > 0 && i % createBlockEvery === 0) {
            await createBlock(200);
            console.log("Generated Msas: ", i);
        }
    }
    await createBlock(750);
    // Create a second block in case there were more than could fit in ^ blocks
    await createBlock();
    const msaIds = await Promise.all(msaPromises)
    console.log("Generated Msas: ", count);
    return msaIds;
}

async function createBlock(wait: number = 300) {
    // Wait ms before creating the block to give the chain time to process the transaction pool
    await new Promise(r => setTimeout(r, wait));
    return firstValueFrom(ExtrinsicHelper.api.rpc.engine.createBlock(true, true));
}

function getMsaFromKey(keys: KeyringPair): Promise<Option<u64>> {
    return ExtrinsicHelper.apiPromise.query.msa.publicKeyToMsaId(keys.address);
}

async function createMsa(keys: KeyringPair): Promise<u64> {
    const result = await ExtrinsicHelper.createMsa(keys).signAndSend();
    const msaRecord = result[1]["msa.MsaCreated"];
    if (msaRecord) return msaRecord.data[0] as u64;

    // This doesn't always work likely due to the load causing the ws subscription to be dropped.
    // So doing a backup call to help
    const tryDirect = await getMsaFromKey(keys);
    if (tryDirect.isSome) return tryDirect.value;

    throw("Failed to get MSA Id...");
}

async function addSigs(msaId: u64, keys: KeyringPair, blockNumber: number, nonce: number): Promise<KeyringPair> {
    let newKeys = createKeys(`${nonce} nonce control key`);

    let defaultPayload: AddKeyData = {};
    defaultPayload.msaId = msaId;
    defaultPayload.newPublicKey = newKeys.publicKey;
    const payload = await generateAddKeyPayload(defaultPayload, 100, blockNumber);

    let addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", payload);

    let ownerSig = signPayloadSr25519(keys, addKeyData);
    let newSig = signPayloadSr25519(newKeys, addKeyData);
    ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, newSig, payload).signAndSend(nonce);

    return newKeys;
}

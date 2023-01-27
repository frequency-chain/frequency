import "@frequency-chain/api-augment";
import assert from "assert";
import { createKeys, createAndFundKeypair, signPayloadSr25519, Sr25519Signature, generateAddKeyPayload, fundKeypair, devAccounts, createAndFundKeypairManual } from "../scaffolding/helpers";
import { KeyringPair } from "@polkadot/keyring/types";
import { AddKeyData, ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { EXISTENTIAL_DEPOSIT } from "../scaffolding/rootHooks";
import { firstValueFrom } from "rxjs";
import { BlockHash, CreatedBlock, Index } from "@polkadot/types/interfaces";
import { BN, u8aToHex } from "@polkadot/util";
import { U64, u64 } from "@polkadot/types";

describe.skip("MSA Initializer Load Tests", function () {
    let keys: KeyringPair;
    let nonce: number = 0;

    beforeEach(async function () {
        const keypair = createKeys();
        keys = devAccounts[0].keys
    });

    // Right now, this test attaches all public keys to a single MSA
    // Our constants limit the number of public keys attached to an MSA
    // to 25 (see the mocks.rs file in case this number changes).
    // 
    // To make this test more accurate, you wili need to create a new MSA
    // id every 25 public keys
    // Something like:
    // 
    // if (i > 0 && i % MAX_KEY_PER_MSA) {
    //    let keys = createAndFundKeyPair()    
    //    [newNonce, msaId] = getMsaId(keys, nonce)
    //
    // }

    it("should successfully create 100,000 signatures within a block", async function () {
        let [newNonce, msaId] = await getMsaId(keys, nonce);
        assert.notEqual(undefined, msaId);

        for (let i = 0; i < 50000; i++) {
            // Our bucket size is 100 blocks.
            // To reach our 50k max (as discussed here: https://github.com/LibertyDSNP/frequency/pull/777/files#diff-6843eb02bb7e86a38f71d2e5c22b6bef714af6c8e01024ab4e4bbfefca831f24R156),
            // we should be inkecting 500 signatures per block. msa_addPublicKeyToMsa adds 2 signatures per call. Therefore, we need to create a new block
            // when the counter hits 250 (500 / 2).
            if (i > 0 && i % 250) {
                await firstValueFrom(ExtrinsicHelper.api.rpc.engine.createBlock(true, true));
            }
            // This call adds 2 signatures
            console.log("Round:", i);
            newNonce = await addSigs(msaId, keys, newNonce);
        }
        await firstValueFrom(ExtrinsicHelper.api.rpc.engine.createBlock(true, true));
    }).timeout(500000)
})

async function getMsaId(keys, nonce): Promise<[number, u64]> {
    const f = ExtrinsicHelper.createMsa(keys);
    await f.signAndSendManual(nonce++);
    let block:CreatedBlock = await firstValueFrom(ExtrinsicHelper.api.rpc.engine.createBlock(true, true));

    const events = await getBlockEvents(block.blockHash);
    let msaId;
    events.forEach((record) => {
        let { event } = record;
        if (event.section === "msa" && event.method === "MsaCreated") { msaId = event.data[0] }
    })
    return [nonce, msaId];
}

async function addSigs(msaId, keys, nonce) {
    let newkeys = await createAndFundKeypairManual(EXISTENTIAL_DEPOSIT, undefined, nonce++);
    // await ExtrinsicHelper.api.rpc.engine.createBlock(true, true);

    let defaultPayload: AddKeyData = {};
    defaultPayload.msaId = msaId;
    defaultPayload.newPublicKey = newkeys.publicKey;
    let payload = await generateAddKeyPayload(defaultPayload);
    let addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", payload);

    let ownerSig = signPayloadSr25519(keys, addKeyData);
    let newSig = signPayloadSr25519(newkeys, addKeyData);
    const addPublicKeyOp = ExtrinsicHelper.addPublicKeyToMsa(keys, ownerSig, newSig, payload);

    addPublicKeyOp.fundAndSendManual(undefined, nonce++);
    // await ExtrinsicHelper.api.rpc.engine.createBlock(true, true);

    return nonce;
}

async function getBlockEvents(blockHash: BlockHash) {
    const blockEvents = await ExtrinsicHelper.api.query.system.events.at(blockHash)
    return firstValueFrom(blockEvents);
}

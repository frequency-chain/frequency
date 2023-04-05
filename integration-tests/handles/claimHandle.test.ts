
import "@frequency-chain/api-augment";
import assert from "assert";
import { createKeys, createAndFundKeypair, signPayloadSr25519, Sr25519Signature, generateAddKeyPayload } from "../scaffolding/helpers";
import { KeyringPair } from "@polkadot/keyring/types";
import { ClaimHandlePayload, ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { Bytes, createType, u8, u64 } from "@polkadot/types";
import { Codec } from "@polkadot/types/types";
import { MultiSignature } from '@polkadot/types/interfaces';
import { CommonPrimitivesHandlesClaimHandlePayload, CommonPrimitivesHandlesRetireHandlePayload } from '@polkadot/types/lookup';

describe.only("Handle Creation", function () {
    let keys: KeyringPair;
    let msaId: u64;
    let authorizedKeys: KeyringPair[] = [];

    before(async function () {
        keys = await createAndFundKeypair();
    });

    describe("claimHandle", function () {

        it("should successfully create an MSA account and claim a handle for it", async function () {
            // Create MSA
            const f = ExtrinsicHelper.createMsa(keys);
            await f.fundOperation();
            const [msaCreatedEvent, chainEvents1] = await f.signAndSend();

            // Claim handle
            const baseHandle = new Bytes(ExtrinsicHelper.api.registry, "HelloWorld");
            const payload = {
                baseHandle: baseHandle 
            };
            const claimHandleData: CommonPrimitivesHandlesClaimHandlePayload = ExtrinsicHelper.api.registry.createType("CommonPrimitivesHandlesClaimHandlePayload", payload);

            let proof = signPayloadSr25519(keys, claimHandleData);
            const claim = ExtrinsicHelper.claimHandle(keys, proof, claimHandleData );
            const [handleClaimedEvent, chainEvents2] = await claim.fundAndSend();

            // Retire handle
            // const retireHandleData: CommonPrimitivesHandlesRetireHandlePayload = ExtrinsicHelper.api.registry.createType("CommonPrimitivesHandlesRetireHandlePayload", retirePayload);
            // const retire = ExtrinsicHelper.retireHandle(keys, proof, retireHandleData );
            // const [handleRetiredEvent, chainEvents3] = await retire.fundAndSend();


        });
    });

});

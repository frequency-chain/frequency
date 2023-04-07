//  Handles test suite
import "@frequency-chain/api-augment";
import assert from "assert";
import {signPayloadSr25519, createProviderKeysAndId, createDelegator} from "../scaffolding/helpers";  // <--- This is the import that is failing
import {KeyringPair} from "@polkadot/keyring/types";
import {Keyring} from "@polkadot/keyring";
import {u8aToHex} from "@polkadot/util";
import {CommonPrimitivesHandlesClaimHandlePayload} from "@polkadot/types/lookup";
import {MessageSourceId} from "@frequency-chain/api-augment/interfaces";
import {ExtrinsicHelper} from "../scaffolding/extrinsicHelpers";
import {Bytes, u16, u32, u64} from "@polkadot/types";

describe("🤝 Handles", () => {
    let msa_id: MessageSourceId;
    let providerId: MessageSourceId;
    let providerKeys: KeyringPair;
    let delegatorKeys: KeyringPair;
    let saved_suffix: u16;
    before(async function () {
        // Create a MSA for the delegator
        [delegatorKeys, msa_id] = await createDelegator();
        assert.notEqual(delegatorKeys, undefined, "setup should populate delegator_key");
        assert.notEqual(msa_id, undefined, "setup should populate msa_id");
    });
    
    describe("@Claim Handle", () => {
        it("should be able to claim a handle", async function () {
            const handle = "test_handle";
            const handle_vec = new Bytes(ExtrinsicHelper.api.registry, handle);
            const payload = {
                baseHandle: handle_vec,
            }
            const claimHandlePayload = ExtrinsicHelper.api.registry.createType("CommonPrimitivesHandlesClaimHandlePayload", payload);
            const claimHandle = ExtrinsicHelper.claimHandle(delegatorKeys, claimHandlePayload);
            const [event] = await claimHandle.fundAndSend();
            assert.notEqual(event, undefined, "claimHandle should return an event");
            if (event && claimHandle.api.events.handles.HandleClaimed.is(event)) {
                let handle = event.data.handle.toString();
                assert.notEqual(handle, "", "claimHandle should emit a handle");
            }
        });
    });

    describe("@Retire Handle", () => {
        it("should be able to retire a handle", async function () {
            let handle_response = await ExtrinsicHelper.getHandleForMSA(msa_id);
            if (!handle_response.isSome) {
                throw new Error("handle_response should be Some");
            }
            let full_handle_state = handle_response.unwrap();
            let suffix_from_state = full_handle_state.suffix;
            let suffix = suffix_from_state.toNumber();
            assert.notEqual(suffix, 0, "suffix should not be 0");
            let full_handle = "test_handle" + "." + suffix.toString();
            const handle_vec = new Bytes(ExtrinsicHelper.api.registry, full_handle);
            const payload = {
                fullHandle: handle_vec,
            }
            const retireHandlePayload = ExtrinsicHelper.api.registry.createType("CommonPrimitivesHandlesRetireHandlePayload", payload);
            const retireHandle = ExtrinsicHelper.retireHandle(delegatorKeys, retireHandlePayload);
            const [event] = await retireHandle.fundAndSend();
            assert.notEqual(event, undefined, "retireHandle should return an event");
            if (event && retireHandle.api.events.handles.HandleRetired.is(event)) {
                let handle = event.data.handle.toString();
                assert.notEqual(handle, "", "retireHandle should return the correct handle");
            }
        });
    });

    describe("@Alt Path: Claim Handle with possible presumptive suffix/RPC test", () => {
       /// Check chain to getNextSuffixesForHandle
       
         it("should be able to claim a handle and check suffix (=suffix_assumed if avaiable on chain)", async function () {
            const handle = "test1";
            let handle_bytes = new Bytes(ExtrinsicHelper.api.registry, handle);
            const request_suffixes = {
                base_handle: handle,
                count: 10,
            };
            /// Get presumptive suffix from chain (rpc)
            let suffix_input_type = ExtrinsicHelper.api.registry.createType("PresumptiveSuffixesRequest", request_suffixes);
            let suffixes_response = await ExtrinsicHelper.getNextSuffixesForHandle(suffix_input_type);
            let resp_base_handle = suffixes_response.base_handle.toString();
            assert.equal(resp_base_handle, handle, "resp_base_handle should be equal to handle");
            let suffix_assumed = suffixes_response.suffixes[0];
            assert.notEqual(suffix_assumed, 0, "suffix_assumed should not be 0");         

            /// Claim handle (extrinsic)
            const payload_ext = {
                baseHandle: handle_bytes,
            };
            const claimHandlePayload = ExtrinsicHelper.api.registry.createType("CommonPrimitivesHandlesClaimHandlePayload", payload_ext);
            const claimHandle = ExtrinsicHelper.claimHandle(delegatorKeys, claimHandlePayload);
            const [event] = await claimHandle.fundAndSend();
            assert.notEqual(event, undefined, "claimHandle should return an event");
            if (event && claimHandle.api.events.handles.HandleClaimed.is(event)) {
                let handle = event.data.handle.toString();
                assert.notEqual(handle, "", "claimHandle should emit a handle");
            }
            // get handle using msa (rpc)
            let handle_response = await ExtrinsicHelper.getHandleForMSA(msa_id);
            if (!handle_response.isSome) {
                throw new Error("handle_response should be Some");
            }
            let full_handle_state = handle_response.unwrap();
            let suffix_from_state = full_handle_state.suffix;
            let suffix = suffix_from_state.toNumber();
            assert.notEqual(suffix, 0, "suffix should not be 0");
            assert.equal(suffix, suffix_assumed, "suffix should be equal to suffix_assumed");
         });
    }); 
});

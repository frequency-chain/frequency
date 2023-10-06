//  Handles test suite
import "@frequency-chain/api-augment";
import assert from "assert";
import { createDelegator } from "../scaffolding/helpers";
import { KeyringPair } from "@polkadot/keyring/types";
import { MessageSourceId } from "@frequency-chain/api-augment/interfaces";
import { ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { Bytes } from "@polkadot/types";
import { getBlockNumber } from "../scaffolding/helpers";
import { hasRelayChain } from "../scaffolding/env";
import { getFundingSource } from "../scaffolding/funding";

describe("🤝 Handles", () => {
    const fundingSource = getFundingSource("handles");

    let msa_id: MessageSourceId;
    let msaOwnerKeys: KeyringPair;
    before(async function () {
        // Create a MSA for the delegator
        [msaOwnerKeys, msa_id] = await createDelegator(fundingSource);
        assert.notEqual(msaOwnerKeys, undefined, "setup should populate delegator_key");
        assert.notEqual(msa_id, undefined, "setup should populate msa_id");
    });

    describe("@Claim Handle", () => {
        it("should be able to claim a handle", async function () {
            const handle = "test_handle";
            let currentBlock = await getBlockNumber();
            const handle_vec = new Bytes(ExtrinsicHelper.api.registry, handle);
            const payload = {
                baseHandle: handle_vec,
                expiration: currentBlock + 10,
            }
            const claimHandlePayload = ExtrinsicHelper.api.registry.createType("CommonPrimitivesHandlesClaimHandlePayload", payload);
            const claimHandle = ExtrinsicHelper.claimHandle(msaOwnerKeys, claimHandlePayload);
            const [event] = await claimHandle.fundAndSend(fundingSource);
            assert.notEqual(event, undefined, "claimHandle should return an event");
            if (event && claimHandle.api.events.handles.HandleClaimed.is(event)) {
                let handle = event.data.handle.toString();
                assert.notEqual(handle, "", "claimHandle should emit a handle");
            }
        });
    });

    describe("@Retire Handle", () => {
        it("should be able to retire a handle", async function () {
            if (hasRelayChain()) {
                this.timeout(250_000 + 12 * 20 * 1000);
            }

            let handle_response = await ExtrinsicHelper.getHandleForMSA(msa_id);
            if (!handle_response.isSome) {
                throw new Error("handle_response should be Some");
            }
            let full_handle_state = handle_response.unwrap();
            let suffix_from_state = full_handle_state.suffix;
            let suffix = suffix_from_state.toNumber();
            assert.notEqual(suffix, 0, "suffix should not be 0");
            assert.notEqual(full_handle_state.canonical_base, undefined, "canonical_base should not be undefined");
            assert.notEqual(full_handle_state.base_handle, undefined, "base_handle should not be undefined");
            let currentBlock = await getBlockNumber();
            await ExtrinsicHelper.runToBlock(currentBlock + 20);

            const retireHandle = ExtrinsicHelper.retireHandle(msaOwnerKeys);
            const [event] = await retireHandle.fundAndSend(fundingSource);
            assert.notEqual(event, undefined, "retireHandle should return an event");
            if (event && retireHandle.api.events.handles.HandleRetired.is(event)) {
                let handle = event.data.handle.toString();
                assert.notEqual(handle, "", "retireHandle should return the correct handle");
            }
        });
    });

    describe("@Alt Path: Claim Handle with possible presumptive suffix/RPC test", () => {
        /// Check chain to getNextSuffixesForHandle

        it("should be able to claim a handle and check suffix (=suffix_assumed if available on chain)", async function () {
            const handle = "test1";
            let handle_bytes = new Bytes(ExtrinsicHelper.api.registry, handle);
            /// Get presumptive suffix from chain (rpc)
            let suffixes_response = await ExtrinsicHelper.getNextSuffixesForHandle(handle, 10);
            let resp_base_handle = suffixes_response.base_handle.toString();
            assert.equal(resp_base_handle, handle, "resp_base_handle should be equal to handle");
            let suffix_assumed = suffixes_response.suffixes[0];
            assert.notEqual(suffix_assumed, 0, "suffix_assumed should not be 0");

            let currentBlock = await getBlockNumber();
            /// Claim handle (extrinsic)
            const payload_ext = {
                baseHandle: handle_bytes,
                expiration: currentBlock + 100,
            };
            const claimHandlePayload = ExtrinsicHelper.api.registry.createType("CommonPrimitivesHandlesClaimHandlePayload", payload_ext);
            const claimHandle = ExtrinsicHelper.claimHandle(msaOwnerKeys, claimHandlePayload);
            const [event] = await claimHandle.fundAndSend(fundingSource);
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

            /// Get MSA from full display handle (rpc)
            let display_handle = handle + "." + suffix;
            let msa_option = await ExtrinsicHelper.getMsaForHandle(display_handle);
            if (!msa_option.isSome) {
                throw new Error("msa_option should be Some");
            }
            let msa_from_handle = msa_option.unwrap();
            assert.equal(msa_from_handle.toString(), msa_id.toString(), "msa_from_handle should be equal to msa_id");
        });
    });

    describe("👇 Negative Test: Early retire handle", () => {
        it("should not be able to retire a handle before expiration", async function () {
            if (hasRelayChain()) {
                this.timeout(250_000 + 12 * 10 * 1000);
            }
            let handle_response = await ExtrinsicHelper.getHandleForMSA(msa_id);
            if (!handle_response.isSome) {
                throw new Error("handle_response should be Some");
            }

            let full_handle_state = handle_response.unwrap();
            let suffix_from_state = full_handle_state.suffix;
            let suffix = suffix_from_state.toNumber();
            assert.notEqual(suffix, 0, "suffix should not be 0");

            let currentBlock = await getBlockNumber();
            await ExtrinsicHelper.runToBlock(currentBlock + 10);
            try {
                const retireHandle = ExtrinsicHelper.retireHandle(msaOwnerKeys);
                const [event] = await retireHandle.fundAndSend(fundingSource);
                assert.equal(event, undefined, "retireHandle should not return an event");
            }
            catch (e) {
                assert.notEqual(e, undefined, "retireHandle should throw an error");
            }
        });
    });

    describe("Suffixes Integrity Check", () => {
        it("should return same suffixes for `abcdefg` from chain as hardcoded", async function () {
            let suffixes = await ExtrinsicHelper.getNextSuffixesForHandle("abcdefg", 10);
            let suffixes_expected = [23, 65, 16, 53, 25, 75, 29, 26, 10, 87];
            let resp_suffixes_number = suffixes.suffixes.map((x) => x.toNumber());
            assert.deepEqual(resp_suffixes_number, suffixes_expected, "suffixes should be equal to suffixes_expected");
        });
    });

    describe("validateHandle basic test", () => {
        it('returns true for good handle, and false for bad handle', async () => {
            let res = await ExtrinsicHelper.validateHandle("Robert`DROP TABLE STUDENTS;--");
            assert.equal(res.toHuman(), false);
            res = await ExtrinsicHelper.validateHandle("Little Bobby Tables")
            assert.equal(res.toHuman(), true);
            res = await ExtrinsicHelper.validateHandle("Bobbay😀😀")
            assert.equal(res.toHuman(), false);
        });
    })
});

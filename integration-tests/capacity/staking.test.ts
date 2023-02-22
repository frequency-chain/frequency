import "@frequency-chain/api-augment";
import { KeyringPair } from "@polkadot/keyring/types";
import { u16, u64 } from "@polkadot/types";
import assert from "assert";
import { AddProviderPayload, Extrinsic, ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { createAndFundKeypair, createKeys, generateDelegationPayload, signPayloadSr25519 } from "../scaffolding/helpers";

describe.only("Capacity Scenario Tests", function () {
    let keys: KeyringPair;
    let otherMsaKeys: KeyringPair;
    let noMsaKeys: KeyringPair;
    let providerKeys: KeyringPair;
    let otherProviderKeys: KeyringPair;
    let schemaId: u16;
    let providerId: u64;
    let otherProviderId: u64;
    let msaId: u64;
    let otherMsaId: u64;

    before(async function () {
        keys = await createAndFundKeypair();

        providerKeys = await createAndFundKeypair();
        let createProviderMsaOp = ExtrinsicHelper.createMsa(providerKeys);
        await createProviderMsaOp.fundAndSend();
        let createProviderOp = ExtrinsicHelper.createProvider(providerKeys, "TestProvider");
        let [providerEvent] = await createProviderOp.fundAndSend();
        assert.notEqual(providerEvent, undefined, "setup should return a ProviderCreated event");
        if (providerEvent && ExtrinsicHelper.api.events.msa.ProviderCreated.is(providerEvent)) {
            providerId = providerEvent.data.providerId;
        }
        assert.notEqual(providerId, undefined, "setup should populate providerId");

        otherProviderKeys = await createAndFundKeypair();
        createProviderMsaOp = ExtrinsicHelper.createMsa(otherProviderKeys);
        await createProviderMsaOp.fundAndSend();
        createProviderOp = ExtrinsicHelper.createProvider(otherProviderKeys, "TestProvider");
        [providerEvent] = await createProviderOp.fundAndSend();
        assert.notEqual(providerEvent, undefined, "setup should return a ProviderCreated event");
        if (providerEvent && ExtrinsicHelper.api.events.msa.ProviderCreated.is(providerEvent)) {
            otherProviderId = providerEvent.data.providerId;
        }
        assert.notEqual(otherProviderId, undefined, "setup should populate providerId");

    });

    describe("stake testing", function () {

        it("should successfully stake the minimum amount", async function () {
            const stakeObj = ExtrinsicHelper.stake(providerKeys, providerId, 1000000);
            const [stakeEvent] = await stakeObj.fundAndSend();
            assert.notEqual(stakeEvent, undefined, "setup should return a Stake event");
        });

        it("should successfully unstake the minimum amount", async function () {
            const stakeObj = ExtrinsicHelper.unstake(providerKeys, providerId, 1000000);
            const [unStakeEvent] = await stakeObj.fundAndSend();
            assert.notEqual(unStakeEvent, undefined, "setup should return an UnStaked event");
        });

        // it("should fail to withdraw the unstaked amount", async function () {
        //     const withdrawObj = ExtrinsicHelper.withdraw_unstaked(providerKeys);
        //     const [withdrawEvent] = await withdrawObj.fundAndSend();
        //     assert.equal(withdrawEvent, "Error::NoUnstakedTokensAvailable", "setup should not return a WithdrawUnstaked event");
        // });
    });

})

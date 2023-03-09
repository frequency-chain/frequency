import "@frequency-chain/api-augment";
import { KeyringPair } from "@polkadot/keyring/types";
import { u64, u16 } from "@polkadot/types";
import assert from "assert";
import { AddProviderPayload, ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { devAccounts, log, createKeys, createAndFundKeypair, createMsaAndProvider,
         generateDelegationPayload, signPayloadSr25519, stakeToProvider }
    from "../scaffolding/helpers";
import { firstValueFrom } from "rxjs";
// import { Call } from "@polkadot/types/interfaces";

// REMOVE: .only for testing
describe.only("Capacity Transaction Tests", function () {
    const TEST_EPOCH_LENGTH = 25;
    let otherProviderKeys: KeyringPair;
    let otherProviderId: u64;
    let stakeKeys: KeyringPair;
    let stakeProviderId: u64;
    let unstakeKeys: KeyringPair;
    let unstakeProviderId: u64;
    let withdrawKeys: KeyringPair;
    let withdrawProviderId: u64;
    let emptyKeys: KeyringPair;
    let emptyProviderId: u64;

    let schemaId: u16;
    let defaultPayload: AddProviderPayload;

    let stakeAmount: bigint = 20000000n;

    before(async function () {
        // Set the Maximum Epoch Length to TEST_EPOCH_LENGTH blocks
        // This will allow us to test the epoch transition logic
        // without having to wait for 100 blocks
        const setEpochLengthOp = ExtrinsicHelper.setEpochLength(devAccounts[0].keys, TEST_EPOCH_LENGTH);
        const [setEpochLengthEvent] = await setEpochLengthOp.sudoSignAndSend();
        if (setEpochLengthEvent &&
            ExtrinsicHelper.api.events.capacity.EpochLengthUpdated.is(setEpochLengthEvent)) {
            const epochLength = setEpochLengthEvent.data.blocks;
            assert.equal(epochLength.toNumber(), TEST_EPOCH_LENGTH, "should set epoch length to TEST_EPOCH_LENGTH blocks");
        }
        else {
            assert.fail("should return an EpochLengthUpdated event");
        }

        // Create and fund a keypair with stakeAmount
        // Use this keypair for stake operations
        stakeKeys = createKeys("StakeKeys");
        stakeProviderId = await createMsaAndProvider(stakeKeys, "StakeProvider", stakeAmount);
        assert.equal(stakeProviderId, 1, "should populate stakeProviderId");

        // Create and fund a keypair with EXISTENTIAL_DEPOSIT
        // Use this keypair for unstake operations
        unstakeKeys = createKeys("UnstakeKeys");
        unstakeProviderId = await createMsaAndProvider(unstakeKeys, "UnstakeProvider");
        assert.equal(unstakeProviderId, 2, "should populate unstakeProviderId");

        // Create and fund a keypair with EXISTENTIAL_DEPOSIT
        // Use this keypair for withdraw operations
        withdrawKeys = createKeys("WithdrawKeys");
        withdrawProviderId = await createMsaAndProvider(withdrawKeys, "WithdrawProvider");
        assert.equal(withdrawProviderId, 3, "should populate withdrawProviderId");

        // Create and fund a keypair with EXISTENTIAL_DEPOSIT
        // Use this keypair for other operations
        otherProviderKeys = createKeys("OtherProviderKeys");
        otherProviderId = await createMsaAndProvider(otherProviderKeys, "OtherProvider", stakeAmount);
        assert.equal(otherProviderId, 4, "should populate otherProviderId");

        // Create a keypair with no tokens
        emptyKeys = await createAndFundKeypair();

        // Create schemas for testing with Grant Delegation to test pay_with_capacity
        // Borrowed from integration-tests/grantDelegation.test.ts, might be a candidate for refactoring
        const createSchemaOp = ExtrinsicHelper.createSchema(stakeKeys, {
            type: "record",
            name: "Post",
            fields: [
                {
                    name: "title",
                    type: {
                        name: "Title",
                        type: "string"
                    }
                },
                {
                    name: "content",
                    type: {
                        name: "Content",
                        type: "string"
                    }
                },
                {
                    name: "fromId",
                    type: {
                        name: "DSNPId",
                        type: "fixed",
                        size: 8,
                    },
                },
                {
                    name: "objectId",
                    type: "DSNPId",
                },
            ]
        }, "AvroBinary", "OnChain");
        const [createSchemaEvent] = await createSchemaOp.fundAndSend();
        assert.notEqual(createSchemaEvent, undefined, "setup should return SchemaCreated event");
        if (createSchemaEvent && ExtrinsicHelper.api.events.schemas.SchemaCreated.is(createSchemaEvent)) {
            schemaId = createSchemaEvent.data[1];
        }
        assert.notEqual(schemaId, undefined, "setup should populate schemaId");
    });
    

    describe("stake-unstake-withdraw_unstaked testing", function () {

        it("should successfully stake the minimum amount for Staked event", async function () {
            await stakeToProvider(stakeKeys, stakeProviderId, 1000000n);

            // Confirm that the tokens were locked in the stakeKeys account using the query API
            const stakedAcctInfo = await ExtrinsicHelper.getAccountInfo(stakeKeys.address);
            assert.equal(stakedAcctInfo.data.miscFrozen, 1000000, "should return an account with 1M miscFrozen balance");
            assert.equal(stakedAcctInfo.data.feeFrozen,  1000000, "should return an account with 1M feeFrozen balance");

            // Confirm that the capacity was added to the stakeProviderId using the query API
            const capacityStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(stakeProviderId))).unwrap();
            assert.equal(capacityStaked.remainingCapacity,   1000000, "should return a capacityLedger with 1M remainingCapacity");
            assert.equal(capacityStaked.totalTokensStaked,   1000000, "should return a capacityLedger with 1M total tokens staked");
            assert.equal(capacityStaked.totalCapacityIssued, 1000000, "should return a capacityLedger with 1M issued capacity");
        });

        it("should successfully unstake the minimum amount", async function () {
            const stakeObj = ExtrinsicHelper.unstake(stakeKeys, stakeProviderId, 1000000);
            const [unStakeEvent] = await stakeObj.fundAndSend();
            assert.notEqual(unStakeEvent, undefined, "should return an UnStaked event");

            if (unStakeEvent && ExtrinsicHelper.api.events.capacity.UnStaked.is(unStakeEvent)) {
                let unstakedCapacity = unStakeEvent.data.capacity;
                assert.equal(unstakedCapacity, 1000000, "should return an UnStaked event with 1000000 reduced capacity");
            }
            else {
                assert.fail("should return an capacity.UnStaked.is(unStakeEvent) event");
            }
            // Confirm that the tokens were unstaked in the stakeProviderId account using the query API
            const capacityStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(stakeProviderId))).unwrap();
            assert.equal(capacityStaked.remainingCapacity,   0, "should return a capacityLedger with 0 remainingCapacity");
            assert.equal(capacityStaked.totalTokensStaked,   0, "should return a capacityLedger with 0 total tokens staked");
            assert.equal(capacityStaked.totalCapacityIssued, 0, "should return a capacityLedger with 0 capacity issued");
        });

        it("withdraws the unstaked amount", async function () {
            // Mine enough blocks to pass the unstake period
            for (let index = 0; index < 2*TEST_EPOCH_LENGTH; index++) {
                await ExtrinsicHelper.createBlock();
            }

            const withdrawObj = ExtrinsicHelper.withdrawUnstaked(stakeKeys);
            const [withdrawEvent] = await withdrawObj.fundAndSend();
            assert.notEqual(withdrawEvent, undefined, "should return a StakeWithdrawn event");

            if (withdrawEvent && ExtrinsicHelper.api.events.capacity.StakeWithdrawn.is(withdrawEvent)) {
                let amount = withdrawEvent.data.amount;
                assert.equal(amount, 1000000, "should return a StakeWithdrawn event with 1M amount");
            }
            // Confirm that the tokens were unstaked in the stakeKeys account using the query API
            const unStakedAcctInfo = await ExtrinsicHelper.getAccountInfo(stakeKeys.address);
            assert.equal(unStakedAcctInfo.data.miscFrozen, 0, "should return an account with 0 miscFrozen balance");
            assert.equal(unStakedAcctInfo.data.feeFrozen,  0, "should return an account with 0 feeFrozen balance");

            // Confirm that the staked capacity was removed from the stakeProviderId account using the query API
            const capacityStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(stakeProviderId))).unwrap();
            assert.equal(capacityStaked.remainingCapacity,   0, "should return a capacityLedger with 0 remainingCapacity");
            assert.equal(capacityStaked.totalTokensStaked,   0, "should return a capacityLedger with 0 total tokens staked");
            assert.equal(capacityStaked.totalCapacityIssued, 0, "should return a capacityLedger with 0 capacity issued");
        });
    });

    describe("increase stake testing", function () {
        it("should successfully increase stake for stakeProviderId", async function () {
            // Now starting from zero again with stakeProviderId, Stake 1M capacity
            await stakeToProvider(stakeKeys, stakeProviderId, 1000000n);

            // Increase stake by 2M capacity
            await stakeToProvider(stakeKeys, stakeProviderId, 2000000n);

            // Confirm that the tokens were staked in the stakeKeys account using the query API
            const stakedAcctInfo = await ExtrinsicHelper.getAccountInfo(stakeKeys.address);
            assert.equal(stakedAcctInfo.data.miscFrozen, 3000000, "should return an account with 3M miscFrozen balance");
            assert.equal(stakedAcctInfo.data.feeFrozen,  3000000, "should return an account with 3M feeFrozen balance");

            // Confirm that the staked capacity was added to the stakeProviderId account using the query API,
            // 1M original stake, + 2M additional stake = 3M total
            const capacityStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(stakeProviderId))).unwrap();
            assert.equal(capacityStaked.remainingCapacity,   3000000, "should return a capacityLedger with 3M remainingCapacity");
            assert.equal(capacityStaked.totalTokensStaked,   3000000, "should return a capacityLedger with 3M total staked");
            assert.equal(capacityStaked.totalCapacityIssued, 3000000, "should return a capacityLedger with 3M capacity issued");
        });

        it("staking to otherProviderId does not change stakeProviderId amounts", async function () {
            // Increase stake by 1000000 capacity to a different provider
            await stakeToProvider(stakeKeys, otherProviderId, 1000000n);

            // Confirm that the staked capacity of the original stakeProviderId account is unchanged
            // stakeProviderId should still have 3M from first test case in this describe.
            // otherProvider should now have 1M
            const origStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(stakeProviderId))).unwrap();
            assert.equal(origStaked.remainingCapacity,   3000000, "should return a capacityLedger with 3M remainingCapacity");
            assert.equal(origStaked.totalTokensStaked,   3000000, "should return a capacityLedger with 3M total tokens staked");
            assert.equal(origStaked.totalCapacityIssued, 3000000, "should return a capacityLedger with 3M capacity issued");

            // Confirm that the staked capacity was added to the otherProviderId account using the query API
            const capacityStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(otherProviderId))).unwrap();
            assert.equal(capacityStaked.remainingCapacity,   1000000, "should return a capacityLedger with 1M remainingCapacity");
            assert.equal(capacityStaked.totalTokensStaked,   1000000, "should return a capacityLedger with 1M total tokens staked");
            assert.equal(capacityStaked.totalCapacityIssued, 1000000, "should return a capacityLedger with 1M capacity issued");
        });
    });

    describe("stake testing-invalid paths", function () {
        it("should fail to stake for InvalidTarget", async function () {
            const failStakeObj = ExtrinsicHelper.stake(stakeKeys, 99, 1000000);
            await assert.rejects(failStakeObj.fundAndSend(), { name: "InvalidTarget" });
        });

        // Need to use a different account that is not already staked
        it("should fail to stake for InsufficientStakingAmount", async function () {
            const failStakeObj = ExtrinsicHelper.stake(otherProviderKeys, otherProviderId, 1000);
            await assert.rejects(failStakeObj.fundAndSend(), { name: "InsufficientStakingAmount" });
        });

        it("should fail to stake for ZeroAmountNotAllowed", async function () {
            const failStakeObj = ExtrinsicHelper.stake(stakeKeys, stakeProviderId, 0);
            await assert.rejects(failStakeObj.fundAndSend(), { name: "ZeroAmountNotAllowed" });
        });
    });

    describe("unstake testing", function () {
        it("should fail to unstake for UnstakedAmountIsZero", async function () {
            const failUnstakeObj = ExtrinsicHelper.unstake(unstakeKeys, unstakeProviderId, 0);
            await assert.rejects(failUnstakeObj.fundAndSend(), { name: "UnstakedAmountIsZero" });
        });
        it("should fail to unstake for StakingAccountNotFound", async function () {
            const failUnstakeObj = ExtrinsicHelper.unstake(otherProviderKeys, unstakeProviderId, 1000000);
            await assert.rejects(failUnstakeObj.fundAndSend(), { name: "StakingAccountNotFound" });
        });
    });

    describe("withdraw_unstaked testing", function () {
        it("should fail to withdraw the unstaked amount", async function () {
            const stakeObj = ExtrinsicHelper.stake(withdrawKeys, withdrawProviderId, 1000000);
            const [stakeEvent] = await stakeObj.fundAndSend();
            assert.notEqual(stakeEvent, undefined, "should return a Stake event");

            const withdrawObj = ExtrinsicHelper.withdrawUnstaked(withdrawKeys);
            assert.rejects(withdrawObj.fundAndSend(), { name: "NoUnstakedTokensAvailable" });
            // Advance to the next block to eliminate nonce collision
            await ExtrinsicHelper.createBlock();
        });
    });

    describe("pay_with_capacity testing", function () {
        it("should successfully stake 9M", async function () {
            // Advance to the next block to eliminate nonce collision
            await ExtrinsicHelper.createBlock();
            await stakeToProvider(stakeKeys, stakeProviderId, 9000000n);
        });
        it("should pay for a transaction with available capacity", async function () {
            // REMOVE:  
            // Confirm that the tokens were staked in the stakeKeys account using the query API
            const stakedAcctInfo = await ExtrinsicHelper.getAccountInfo(stakeKeys.address);
            log("DBG:stakedAcctInfo.data.miscFrozen: ", stakedAcctInfo.data.miscFrozen.toBigInt());
            log("DBG:stakedAcctInfo.data.feeFrozen: ", stakedAcctInfo.data.feeFrozen.toBigInt());
            // assert.equal(stakedAcctInfo.data.miscFrozen, 4000000, "should return an account with 4000000 miscFrozen balance");
            // assert.equal(stakedAcctInfo.data.feeFrozen,  4000000, "should return an account with 4000000 feeFrozen balance");
            // Confirm that the capacity ledger is not empty
            const origStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(stakeProviderId))).unwrap();
            log("DBG:origStaked.remainingCapacity: ", origStaked.remainingCapacity.toBigInt());
            // assert.equal(origStaked.remainingCapacity,   3000000, "should return a capacityLedger with 3M remainingCapacity");
            // assert.equal(origStaked.totalTokensStaked,   3000000, "should return a capacityLedger with 3M total tokens staked");
            // assert.equal(origStaked.totalCapacityIssued, 3000000, "should return a capacityLedger with 3M capacity issued");

            const payload = await generateDelegationPayload({
                authorizedMsaId: stakeProviderId,
                schemaIds: [schemaId],
            });
            const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);
            const grantDelegationOp = ExtrinsicHelper.grantDelegation(otherProviderKeys, stakeKeys, 
                signPayloadSr25519(otherProviderKeys, addProviderData), payload);
            const [grantDelegationEvent] = await grantDelegationOp.payWithCapacity();
            log("DBG:grantDelegationEvent: ", grantDelegationEvent);
            if (grantDelegationEvent && 
                ExtrinsicHelper.api.events.msa.DelegationGranted.is(grantDelegationEvent)) {
                log("DBG:grantDelegationEvent: ", grantDelegationEvent);
            }
            else {
                assert.fail("should return a DelegationGranted event");
            }
            
            // const addMessageOp = ExtrinsicHelper.addOnChainMessage(stakeKeys, 1, "test message");
            // const [addMessageEvent] = await addMessageOp.payWithCapacity();
            // assert.notEqual(addMessageEvent, undefined, "should return an event");
            // await assert.rejects(grantDelegationOp.payWithCapacity(), { name: "Error", message: 
            //     "1010: Invalid Transaction: Inability to pay some fees , e.g. account balance too low" });
        });
        it("should fail to pay for a non-capacity transaction with available capacity", async function () {
            // REMOVE:
            // Confirm that the capacity ledger is not empty
            const origStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(stakeProviderId))).unwrap();
            log("DBG:origStaked.remainingCapacity: ", origStaked.remainingCapacity.toBigInt());
            // assert.equal(origStaked.remainingCapacity,   134836, "should return a capacityLedger with 134836 remainingCapacity:2");
            // assert.equal(origStaked.totalTokensStaked,   3000000, "should return a capacityLedger with 3M total tokens staked");
            // assert.equal(origStaked.totalCapacityIssued, 3000000, "should return a capacityLedger with 3M capacity issued");

            // stake is not capacity eligible
            const increaseStakeObj = ExtrinsicHelper.stake(stakeKeys, otherProviderId, 1000000);
            await assert.rejects(increaseStakeObj.payWithCapacity(), { name: "RpcError", message: 
                "1010: Invalid Transaction: Custom error: 0" });
        });
        it("should fail to pay for a transaction with no msaId", async function () {
            const payload = await generateDelegationPayload({
                authorizedMsaId: withdrawProviderId,
                schemaIds: [schemaId],
            });
            const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);
            const grantDelegationOp = ExtrinsicHelper.grantDelegation(emptyKeys, withdrawKeys, 
                signPayloadSr25519(emptyKeys, addProviderData), payload);
            await assert.rejects(grantDelegationOp.payWithCapacity(), { name: "RpcError", message: 
                "1010: Invalid Transaction: Custom error: 1" });
        });
    });
    describe("empty capacity testing", function () {
        // When a user does not have enough capacity to pay for the transaction fee
        // and is NOT eligible to replenish Capacity, it should error and be dropped
        // from the transaction pool.
        it("should fail to pay for a transaction with empty capacity", async function () {
            const payload = await generateDelegationPayload({
                authorizedMsaId: withdrawProviderId,
                schemaIds: [schemaId],
            });
            const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);
            const grantDelegationOp = ExtrinsicHelper.grantDelegation(otherProviderKeys, withdrawKeys, 
                signPayloadSr25519(withdrawKeys, addProviderData), payload);
            await assert.rejects(grantDelegationOp.payWithCapacity(), { name: "RpcError", message: 
                "1010: Invalid Transaction: Inability to pay some fees , e.g. account balance too low" });
        });
    });
})

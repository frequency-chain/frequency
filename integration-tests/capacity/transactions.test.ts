import "@frequency-chain/api-augment";
import { KeyringPair } from "@polkadot/keyring/types";
import { u64, u16 } from "@polkadot/types";
import assert from "assert";
import { ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { devAccounts, log, createKeys, createAndFundKeypair, createMsaAndProvider,
         generateDelegationPayload, signPayloadSr25519, stakeToProvider, fundKeypair }
    from "../scaffolding/helpers";
import { firstValueFrom } from "rxjs";
import { AVRO_GRAPH_CHANGE } from "../schemas/fixtures/avroGraphChangeSchemaType";

describe("Capacity Transaction Tests", function () {
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
    let noProviderKeys: KeyringPair;
    let delegatorKeys: KeyringPair;
    let delegatorProviderId: u64;
    let schemaId: u16;

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

        // Create a keypair with no msaId
        emptyKeys = await createAndFundKeypair();

        // Create and fund a keypair with EXISTENTIAL_DEPOSIT
        // Use this keypair for delegator operations
        delegatorKeys = createKeys("OtherProviderKeys");
        delegatorProviderId = await createMsaAndProvider(delegatorKeys, "Delegator", stakeAmount);
        assert.equal(delegatorProviderId, 5, "should populate delegatorProviderId");

        // Create a keypair with msaId, but no provider
        noProviderKeys = createKeys("NoProviderKeys");
        await fundKeypair(devAccounts[0].keys, noProviderKeys, stakeAmount);
        const createMsaOp = ExtrinsicHelper.createMsa(noProviderKeys);
        const [MsaCreatedEvent] = await createMsaOp.fundAndSend();
        assert.notEqual(MsaCreatedEvent, undefined, "should have returned MsaCreated event");

        // Create schemas for testing with Grant Delegation to test pay_with_capacity
        const createSchemaOp = ExtrinsicHelper.createSchema(stakeKeys, AVRO_GRAPH_CHANGE, "AvroBinary", "OnChain");
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
        });
    });

    describe("pay_with_capacity testing", function () {
        it("should pay for a transaction with available capacity", async function () {
            // Advance to the next block to eliminate a nonce conflict from previous tests
            await ExtrinsicHelper.createBlock();
            await stakeToProvider(stakeKeys, stakeProviderId, 9000000n);
            // grantDelegation costs about 2.8M and is the capacity eligible
            // transaction used for these tests.
            const payload = await generateDelegationPayload({
                authorizedMsaId: stakeProviderId,
                schemaIds: [schemaId],
            });
            const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);
            const grantDelegationOp = ExtrinsicHelper.grantDelegation(otherProviderKeys, stakeKeys, 
                signPayloadSr25519(otherProviderKeys, addProviderData), payload);
            const [grantDelegationEvent, chainEvents] = await grantDelegationOp.payWithCapacity();
            if (grantDelegationEvent && 
                !(ExtrinsicHelper.api.events.msa.DelegationGranted.is(grantDelegationEvent))) {
                assert.fail("should return a DelegationGranted event");
            }
            // When Capacity has withdrawn an event CapacityWithdrawn is emitted.
            if (chainEvents && chainEvents["capacity.CapacityWithdrawn"].isEmpty) {
                assert.fail("chainEvents should contain a CapacityWithdrawn event");
            }
        });
        // When a user attempts to pay for a non-capacity transaction with Capacity,
        // it should error and drop the transaction from the transaction-pool.
        it("should fail to pay for a non-capacity transaction with available capacity", async function () {
            // stake is not capacity eligible
            const increaseStakeObj = ExtrinsicHelper.stake(stakeKeys, otherProviderId, 1000000);
            await assert.rejects(increaseStakeObj.payWithCapacity(), { name: "RpcError", message: 
                "1010: Invalid Transaction: Custom error: 0" });
        });
        it("should fail to pay for a transaction with no msaId", async function () {
            const payload = await generateDelegationPayload({
                authorizedMsaId: delegatorProviderId,
                schemaIds: [schemaId],
            });
            const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);
            const grantDelegationOp = ExtrinsicHelper.grantDelegation(delegatorKeys, emptyKeys, 
                signPayloadSr25519(delegatorKeys, addProviderData), payload);
            await assert.rejects(grantDelegationOp.payWithCapacity(), { name: "RpcError", message: 
                "1010: Invalid Transaction: Custom error: 1" });
        });
        it("should fail to pay for a transaction with no provider", async function () {
            // When a user is not a registered provider and attempts to pay with Capacity,
            // it should error with InvalidTransaction::Payment, which is a 1010 error, Inability to pay some fees.
            const payload = await generateDelegationPayload({
                authorizedMsaId: delegatorProviderId,
                schemaIds: [schemaId],
            });
            const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);
            const grantDelegationOp = ExtrinsicHelper.grantDelegation(delegatorKeys, noProviderKeys, 
                signPayloadSr25519(delegatorKeys, addProviderData), payload);
            await assert.rejects(grantDelegationOp.payWithCapacity(), { name: "RpcError", message: 
                "1010: Invalid Transaction: Inability to pay some fees , e.g. account balance too low" });
        });
        // A registered provider with Capacity but no tokens associated with the
        // key should still be able to use polkadot UI to submit a capacity transaction.
        // *All accounts will have at least an EXISTENTIAL_DEPOSIT = 1M.
        // This test may still be valid if the txn cost is greater than 1M.
        it("should pay for a transaction with available capacity", async function () {
            // Stake enough to cover the 2.8M grantDelegation cost
            await stakeToProvider(stakeKeys, unstakeProviderId, 3000000n);
            // grantDelegation costs about 2.8M and is the the capacity eligible
            // transaction used for these tests.
            const payload = await generateDelegationPayload({
                authorizedMsaId: stakeProviderId,
                schemaIds: [schemaId],
            });
            const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);
            const grantDelegationOp = ExtrinsicHelper.grantDelegation(otherProviderKeys, unstakeKeys, 
                signPayloadSr25519(otherProviderKeys, addProviderData), payload);
            const [grantDelegationEvent, chainEvents] = await grantDelegationOp.payWithCapacity();
            if (grantDelegationEvent && 
                !(ExtrinsicHelper.api.events.msa.DelegationGranted.is(grantDelegationEvent))) {
                assert.fail("should return a DelegationGranted event");
            }
            // When Capacity has withdrawn an event CapacityWithdrawn is emitted.
            if (chainEvents && chainEvents["capacity.CapacityWithdrawn"].isEmpty) {
                assert.fail("chainEvents should return a CapacityWithdrawn event");
            }
        });
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

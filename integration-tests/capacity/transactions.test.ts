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

        // Create and fund a keypair with EXISTENTIAL_DEPOSIT
        // Use this keypair for unstake operations
        unstakeKeys = createKeys("UnstakeKeys");
        unstakeProviderId = await createMsaAndProvider(unstakeKeys, "UnstakeProvider");

        // Create and fund a keypair with EXISTENTIAL_DEPOSIT
        // Use this keypair for withdraw operations
        withdrawKeys = createKeys("WithdrawKeys");
        withdrawProviderId = await createMsaAndProvider(withdrawKeys, "WithdrawProvider");

        // Create and fund a keypair with EXISTENTIAL_DEPOSIT
        // Use this keypair for other operations
        otherProviderKeys = createKeys("OtherProviderKeys");
        otherProviderId = await createMsaAndProvider(otherProviderKeys, "OtherProvider", stakeAmount);

        // Create a keypair with no msaId
        emptyKeys = await createAndFundKeypair();

        // Create and fund a keypair with EXISTENTIAL_DEPOSIT
        // Use this keypair for delegator operations
        delegatorKeys = createKeys("OtherProviderKeys");
        delegatorProviderId = await createMsaAndProvider(delegatorKeys, "Delegator", stakeAmount);

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

    describe("pay_with_capacity testing", function () {
        it("should pay for a transaction with available capacity", async function () {
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

import "@frequency-chain/api-augment";
import { KeyringPair } from "@polkadot/keyring/types";
import { u64, u16, u128 } from "@polkadot/types";
import assert from "assert";
import { AddKeyData, EventMap, ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import {
    devAccounts, createKeys, createAndFundKeypair, createMsaAndProvider,
    generateDelegationPayload, signPayloadSr25519, stakeToProvider, fundKeypair,
    TEST_EPOCH_LENGTH,
    setEpochLength,
    generateAddKeyPayload
} from "../scaffolding/helpers";
import { AVRO_GRAPH_CHANGE } from "../schemas/fixtures/avroGraphChangeSchemaType";
import { firstValueFrom } from "rxjs";

function assertEvent(events: EventMap, eventName: string) {
    assert(events.hasOwnProperty(eventName));
}

describe("Capacity Transactions", function () {
    let CENTS = 1000000n;
    let DOLLARS = 100n * CENTS;
    let stakeAmount: bigint = 200n * DOLLARS;

    before(async function () {
        await setEpochLength(devAccounts[0].keys, TEST_EPOCH_LENGTH);
    });

    describe("pay_with_capacity", function () {
        describe("when caller has a Capacity account", async function () {
            let stakeKeys: KeyringPair;
            let stakeProviderId: u64;
            let schemaId: u16;
            const amountStaked = 9n * CENTS;

            beforeEach(async function () {
                stakeKeys = createKeys("StakeKeys");
                stakeProviderId = await createMsaAndProvider(stakeKeys, "StakeProvider", stakeAmount);

                // Create schemas for testing with Grant Delegation to test pay_with_capacity
                const createSchemaOp = ExtrinsicHelper.createSchema(stakeKeys, AVRO_GRAPH_CHANGE, "AvroBinary", "OnChain");
                const [createSchemaEvent] = await createSchemaOp.fundAndSend();

                assert.notEqual(createSchemaEvent, undefined, "setup should return SchemaCreated event");

                if (createSchemaEvent && ExtrinsicHelper.api.events.schemas.SchemaCreated.is(createSchemaEvent)) {
                    schemaId = createSchemaEvent.data[1];
                }

                assert.notEqual(schemaId, undefined, "setup should populate schemaId");
            });

            it("successfully pays with Capacity for eligible transaction - grantDelegation", async function () {
                await assert.doesNotReject(stakeToProvider(stakeKeys, stakeProviderId, amountStaked));

                let delegatorKeys = createKeys("userKeys");
                await fundKeypair(devAccounts[0].keys, delegatorKeys, 100n * DOLLARS);

                let [MsaCreatedEvent] = await ExtrinsicHelper.createMsa(delegatorKeys).signAndSend();
                assert.notEqual(MsaCreatedEvent, undefined, "should have returned MsaCreated event");

                const payload = await generateDelegationPayload({
                    authorizedMsaId: stakeProviderId,
                    schemaIds: [schemaId],
                });
                const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);
                const grantDelegationOp = ExtrinsicHelper.grantDelegation(delegatorKeys, stakeKeys,
                    signPayloadSr25519(delegatorKeys, addProviderData), payload);

                const [grantDelegationEvent, chainEvents] = await grantDelegationOp.payWithCapacity();

                if (grantDelegationEvent &&
                    !(ExtrinsicHelper.api.events.msa.DelegationGranted.is(grantDelegationEvent))) {
                    assert.fail("should return a DelegationGranted event");
                }

                let fee: u128;
                if (chainEvents["capacity.CapacityWithdrawn"] && 
                    ExtrinsicHelper.api.events.capacity.CapacityWithdrawn.is(chainEvents["capacity.CapacityWithdrawn"])) 
                {
                    fee = chainEvents["capacity.CapacityWithdrawn"].data.amount;
                }
                else {
                    assert.fail("should return a CapacityWithdrawn event");
                }
                const remainingCapacity = amountStaked - fee.toBigInt();

                // Check for remaining capacity to be reduced by correct amount
                const capacityStaked = (await firstValueFrom(ExtrinsicHelper.api.query.capacity.capacityLedger(stakeProviderId))).unwrap();
                assert.equal(capacityStaked.remainingCapacity, remainingCapacity, 
                    `should return a capacityLedger with ${remainingCapacity} remainingCapacity`);
                assert.equal(capacityStaked.totalTokensStaked, amountStaked, 
                    `should return a capacityLedger with ${amountStaked} total tokens staked`);
                assert.equal(capacityStaked.totalCapacityIssued, amountStaked, 
                    `should return a capacityLedger with ${amountStaked} total capacity issued`);
            });

            // When a user attempts to pay for a non-capacity transaction with Capacity,
            // it should error and drop the transaction from the transaction-pool.
            it("fails to pay with Capacity for a non-capacity transaction", async function () {
                let providerId: u64 = new u64(ExtrinsicHelper.api.registry, 0);

                const increaseStakeObj = ExtrinsicHelper.stake(stakeKeys, providerId, 1n * CENTS);

                await assert.rejects(increaseStakeObj.payWithCapacity(), {
                    name: "RpcError", message:
                        "1010: Invalid Transaction: Custom error: 0"
                });
            });

            // When a user does not have enough capacity to pay for the transaction fee
            // and is NOT eligible to replenish Capacity, it should error and be dropped
            // from the transaction pool.
            it("fails to pay for a transaction with empty capacity", async function () {
                let providerKeys = createKeys("providerKeys");
                let _providerId = await createMsaAndProvider(providerKeys, "UnstakeProvider");

                let delegatorKeys = createKeys("userKeys");
                let _userMsaId = await ExtrinsicHelper.createMsa(delegatorKeys);

                let providerId: u64 = new u64(ExtrinsicHelper.api.registry, 0);
                const payload = await generateDelegationPayload({
                    authorizedMsaId: providerId,
                    schemaIds: [schemaId],
                });
                const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);
                const grantDelegationOp = ExtrinsicHelper.grantDelegation(delegatorKeys, providerKeys,
                    signPayloadSr25519(providerKeys, addProviderData), payload);

                await assert.rejects(grantDelegationOp.payWithCapacity(), {
                    name: "RpcError", message:
                        "1010: Invalid Transaction: Inability to pay some fees , e.g. account balance too low"
                });
            });

            // *All keys should have at least an EXISTENTIAL_DEPOSIT = 1M.
            it("fails to pay for transaction when key does has not met the min deposit", async function () {
                let newProviderKeys = createKeys("newKeys");

                await assert.doesNotReject(stakeToProvider(stakeKeys, stakeProviderId, 1n * DOLLARS));

                // Add new key
                let newKeyPayload: AddKeyData = await generateAddKeyPayload({ msaId: new u64(ExtrinsicHelper.api.registry, stakeProviderId), newPublicKey: newProviderKeys.publicKey });
                let addKeyData = ExtrinsicHelper.api.registry.createType("PalletMsaAddKeyData", newKeyPayload);

                let ownerSig = signPayloadSr25519(stakeKeys, addKeyData);
                let newSig = signPayloadSr25519(newProviderKeys, addKeyData);
                const addPublicKeyOp = ExtrinsicHelper.addPublicKeyToMsa(stakeKeys, ownerSig, newSig, newKeyPayload);

                const [publicKeyEvents] = await addPublicKeyOp.fundAndSend();
                assert.notEqual(publicKeyEvents, undefined, 'should have added public key');

                let delegatorKeys = createKeys("delegatorKeys");
                await fundKeypair(devAccounts[0].keys, delegatorKeys, 100n * DOLLARS);
                const createMsaOp = ExtrinsicHelper.createMsa(delegatorKeys);
                const [MsaCreatedEvent] = await createMsaOp.fundAndSend();
                assert.notEqual(MsaCreatedEvent, undefined, "should have returned MsaCreated event");

                const payload = await generateDelegationPayload({
                    authorizedMsaId: stakeProviderId,
                    schemaIds: [schemaId],
                });
                const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);
                const grantDelegationOp = ExtrinsicHelper.grantDelegation(delegatorKeys, newProviderKeys,
                    signPayloadSr25519(delegatorKeys, addProviderData), payload);

                await assert.rejects(grantDelegationOp.payWithCapacity(), {
                    name: 'RpcError',
                    message: /Custom error: 3/,
                })
            });
        });

        describe("when caller does not have a Capacity account", async function () {
            let delegatorKeys: KeyringPair;
            let delegatorProviderId: u64;
            let schemaId: u16;

            beforeEach(async function () {
                // Create and fund a keypair with EXISTENTIAL_DEPOSIT
                // Use this keypair for delegator operations
                delegatorKeys = createKeys("OtherProviderKeys");
                delegatorProviderId = await createMsaAndProvider(delegatorKeys, "Delegator", stakeAmount);
                schemaId = new u16(ExtrinsicHelper.api.registry, 0);
            });

            describe("but has an MSA account that has not been registered as a Provider", async function () {
                it("fails to pay for a transaction", async function () {
                    // Create a keypair with msaId, but no provider
                    let noProviderKeys = createKeys("NoProviderKeys");
                    await fundKeypair(devAccounts[0].keys, noProviderKeys, stakeAmount);
                    const createMsaOp = ExtrinsicHelper.createMsa(noProviderKeys);

                    const [MsaCreatedEvent] = await createMsaOp.fundAndSend();
                    assert.notEqual(MsaCreatedEvent, undefined, "should have returned MsaCreated event");

                    // When a user is not a registered provider and attempts to pay with Capacity,
                    // it should error with InvalidTransaction::Payment, which is a 1010 error, Inability to pay some fees.
                    const payload = await generateDelegationPayload({
                        authorizedMsaId: delegatorProviderId,
                        schemaIds: [schemaId],
                    });
                    const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);
                    const grantDelegationOp = ExtrinsicHelper.grantDelegation(delegatorKeys, noProviderKeys,
                        signPayloadSr25519(delegatorKeys, addProviderData), payload);

                    await assert.rejects(grantDelegationOp.payWithCapacity(), {
                        name: "RpcError", message:
                            "1010: Invalid Transaction: Inability to pay some fees , e.g. account balance too low"
                    });
                });
            });

            describe("and does not have an MSA account associated to signing keys", async function () {
                it("fails to pay for a transaction", async function () {
                    let emptyKeys = await createAndFundKeypair();

                    const payload = await generateDelegationPayload({
                        authorizedMsaId: delegatorProviderId,
                        schemaIds: [schemaId],
                    });
                    const addProviderData = ExtrinsicHelper.api.registry.createType("PalletMsaAddProvider", payload);
                    const grantDelegationOp = ExtrinsicHelper.grantDelegation(delegatorKeys, emptyKeys,
                        signPayloadSr25519(delegatorKeys, addProviderData), payload);

                    await assert.rejects(grantDelegationOp.payWithCapacity(), {
                        name: "RpcError", message:
                            "1010: Invalid Transaction: Custom error: 1"
                    });
                });
            });
        });
    });
});

import { ExtrinsicHelper } from "./extrinsicHelpers";

export let EXISTENTIAL_DEPOSIT: bigint;

export const mochaHooks = {
    async beforeAll() {
        await ExtrinsicHelper.initialize();

        EXISTENTIAL_DEPOSIT = ExtrinsicHelper.api.consts.balances.existentialDeposit.toBigInt();
    },
    async afterAll() {
        await ExtrinsicHelper.api.disconnect();
        await ExtrinsicHelper.apiPromise.disconnect();
    }
}

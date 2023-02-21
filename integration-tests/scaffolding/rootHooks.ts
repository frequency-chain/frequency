import { ExtrinsicHelper } from "./extrinsicHelpers";
import { createKeys } from "./apiConnection";
import { devAccounts } from "./helpers";

export let EXISTENTIAL_DEPOSIT: bigint;

exports.mochaHooks = {
    async beforeAll() {
        await ExtrinsicHelper.initialize();
        for (const uri of ["//Alice", "//Bob", "//Charlie", "//Dave", "//Eve", "//Ferdie"]) {
            devAccounts.push({
                uri,
                keys: createKeys(uri),
            });
        }

        EXISTENTIAL_DEPOSIT = ExtrinsicHelper.api.consts.balances.existentialDeposit.toBigInt();
        console.log({EXISTENTIAL_DEPOSIT})
    },
    async afterAll() {
        await ExtrinsicHelper.api.disconnect();
    }
}

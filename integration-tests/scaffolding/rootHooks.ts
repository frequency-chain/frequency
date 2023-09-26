import { ExtrinsicHelper } from "./extrinsicHelpers";
import { createKeys } from "./apiConnection";
import { CHAIN_ENVIRONMENT, devAccounts, rococoAccounts } from "./helpers";

export let EXISTENTIAL_DEPOSIT: bigint;

export const mochaHooks = {
    async beforeAll() {

        // Default Timeout of 10 seconds
        this.timeout(10_000);

        await ExtrinsicHelper.initialize();

        if (process.env.CHAIN_ENVIRONMENT === CHAIN_ENVIRONMENT.ROCOCO_TESTNET) {
            const seed_phrase = process.env.FUNDING_ACCOUNT_SEED_PHRASE;

            if (seed_phrase === undefined) {
                console.error("FUNDING_ACCOUNT_SEED_PHRASE must not be undefined when CHAIN_ENVIRONMENT is \"rococo\"");
                process.exit(1);
            }

            rococoAccounts.push({
                uri: "RococoTestRunnerAccount",
                keys: createKeys(seed_phrase),
        });
        } else {
            for (const uri of ["//Alice", "//Bob", "//Charlie", "//Dave", "//Eve", "//Ferdie"]) {
                devAccounts.push({
                    uri,
                    keys: createKeys(uri),
                });
            }
        }


        EXISTENTIAL_DEPOSIT = ExtrinsicHelper.api.consts.balances.existentialDeposit.toBigInt();
    },
    async afterAll() {
        await ExtrinsicHelper.api.disconnect();
        await ExtrinsicHelper.apiPromise.disconnect();
    }
}

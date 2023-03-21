import "@frequency-chain/api-augment";
import assert from "assert";
import { createAndFundKeypair } from "../scaffolding/helpers";
import { KeyringPair } from "@polkadot/keyring/types";
import { ExtrinsicHelper, ReleaseSchedule } from "../scaffolding/extrinsicHelpers";
import { devAccounts } from "../scaffolding/helpers";


const DOLLARS: number = 100000000; // 100_000_000

export function numberOfblockBetweenVest(): number {
    // 12 second block production
    const MILLISECS_PER_BLOCK = 12000;
    const MILLISECS_PER_SECOND = 60000;

    const MINUTES = MILLISECS_PER_SECOND / MILLISECS_PER_BLOCK;
    const HOURS = MINUTES * 60;
    const DAYS = HOURS * 24;
    const WEEK = DAYS * 7;

    /// update
    const period = WEEK * 24;
    return period;
}

export function calculateReleaseSchedule(amount: number | bigint): ReleaseSchedule {
    const start = 0;
    const period = numberOfblockBetweenVest();
    const periodCount = 4;

    const perPeriod = BigInt(amount) / (BigInt(periodCount));

    return {
        start,
        period,
        periodCount,
        perPeriod,
    };
}

describe("TimeRelease", function () {
    let vesterKeys: KeyringPair;

    before(async function () {
        vesterKeys = await createAndFundKeypair();
    });

    describe("vested transfer and claim flow", function () {
        it("creates a vested transfer", async function () {
            let sudoKeys: KeyringPair = devAccounts[0].keys;
            let amount = 100000n * BigInt(DOLLARS);
            let schedule: ReleaseSchedule = calculateReleaseSchedule(amount);

            const vestedTransferTx = ExtrinsicHelper.timeReleaseTransfer(sudoKeys, vesterKeys, schedule);
            const [event, eventMap] = await vestedTransferTx.signAndSend();
            assert.notEqual(event, undefined, "should have returned ReleaseScheduleAdded event");
        });
    });
})

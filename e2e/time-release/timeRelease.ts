import "@frequency-chain/api-augment";
import assert from "assert";
import { createAndFundKeypair } from "../scaffolding/helpers";
import { KeyringPair } from "@polkadot/keyring/types";
import { ExtrinsicHelper, ReleaseSchedule } from "../scaffolding/extrinsicHelpers";
import { getFundingSource } from "../scaffolding/funding";

const DOLLARS: number = 100000000; // 100_000_000

export function getBlocksInMonthPeriod(blockTime, periodInMonths) {
    const secondsPerMonth = 2592000; // Assuming 30 days in a month

    // Calculate the number of blocks in the given period
    const blocksInPeriod = Math.floor((periodInMonths * secondsPerMonth) / blockTime);

    return blocksInPeriod;
  }

export function calculateReleaseSchedule(amount: number | bigint): ReleaseSchedule {
    const start = 0;
    const period = getBlocksInMonthPeriod(6, 4);
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
    const sourceKey: KeyringPair = getFundingSource("time-release");

    before(async function () {
        vesterKeys = await createAndFundKeypair(sourceKey, 50_000_000n);
    });

    describe("vested transfer and claim flow", function () {
        it("creates a vested transfer", async function () {
            let amount = 100000n * BigInt(DOLLARS);
            let schedule: ReleaseSchedule = calculateReleaseSchedule(amount);

            const vestedTransferTx = ExtrinsicHelper.timeReleaseTransfer(sourceKey, vesterKeys, schedule);
            const [event, eventMap] = await vestedTransferTx.signAndSend();
            assert.notEqual(event, undefined, "should have returned ReleaseScheduleAdded event");
        });
    });
})

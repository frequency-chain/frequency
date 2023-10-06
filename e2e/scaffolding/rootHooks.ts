import { cryptoWaitReady } from "@polkadot/util-crypto";
import { ExtrinsicHelper } from "./extrinsicHelpers";

export const mochaHooks = {
  async beforeAll() {
    await cryptoWaitReady();
    await ExtrinsicHelper.initialize();
  },
  async afterAll() {
    await ExtrinsicHelper.api.disconnect();
    await ExtrinsicHelper.apiPromise.disconnect();
  }
}

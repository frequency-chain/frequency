import { cryptoWaitReady } from "@polkadot/util-crypto";
import { ExtrinsicHelper } from "./extrinsicHelpers";
import { drainFundedKeys } from "./helpers";
import { getRootFundingSource } from "./funding";

export const mochaHooks = {
  async beforeAll() {
    await cryptoWaitReady();
    await ExtrinsicHelper.initialize();
  },
  async afterAll() {
    // Any key created using helpers `createKeys` is kept in the module
    // then any value remaining is drained here at the end
    const rootAddress = getRootFundingSource().keys.address;
    await drainFundedKeys(rootAddress);
    await ExtrinsicHelper.api.disconnect();
    await ExtrinsicHelper.apiPromise.disconnect();
  }
}

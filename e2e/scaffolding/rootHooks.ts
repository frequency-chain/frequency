// These run once per TEST FILE

import { cryptoWaitReady } from "@polkadot/util-crypto";
import { ExtrinsicHelper } from "./extrinsicHelpers";
import { drainFundedKeys } from "./helpers";
import { getRootFundingSource } from "./funding";

export const mochaHooks = {
  async beforeAll() {
    try {
      await cryptoWaitReady();
      await ExtrinsicHelper.initialize();
    } catch(e) {
      console.error("Failed to run beforeAll root hook: ", e);
    }
  },
  async afterAll() {
    try {
      // Any key created using helpers `createKeys` is kept in the module
      // then any value remaining is drained here at the end
      const rootAddress = getRootFundingSource().keys.address;
      await drainFundedKeys(rootAddress);
      await ExtrinsicHelper.api.disconnect();
      await ExtrinsicHelper.apiPromise.disconnect();
    } catch(e) {
      console.error("Failed to run afterAll root hook: ", e);
    }
  }
}

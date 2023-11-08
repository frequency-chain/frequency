// These run once per TEST FILE

import { cryptoWaitReady } from "@polkadot/util-crypto";
import { ExtrinsicHelper } from "./extrinsicHelpers";
import { drainFundedKeys } from "./helpers";
import { getRootFundingSource } from "./funding";

// Make sure that we can serialize BigInts for Mocha
(BigInt.prototype as any).toJSON = function () {
  return this.toString();
};

export const mochaHooks = {
  async beforeAll() {
    try {
      await cryptoWaitReady();
      await ExtrinsicHelper.initialize();
    } catch (e) {
      console.error('Failed to run beforeAll root hook: ', this.test.parent.suites[0].title, e);
    }
  },
  async afterAll() {
    const testSuite = this.test.parent.suites[0].title;
    console.log("Starting ROOT hook shutdown", testSuite)
    try {
      // Any key created using helpers `createKeys` is kept in the module
      // then any value remaining is drained here at the end
      const rootAddress = getRootFundingSource().keys.address;
      await drainFundedKeys(rootAddress);
      console.log("ENDING ROOT hook shutdown", testSuite)
    } catch (e) {
      console.error('Failed to run afterAll root hook: ', testSuite, e);
    }
  }
}

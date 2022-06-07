import assert from "assert";
import { rpc, types } from "../index.js";

describe("index", function () {
  it("should have rpc calls", function () {
    const keys = Object.keys(rpc);
    assert(keys.length > 0);
  });
  it("should have types", function () {
    const keys = Object.keys(types);
    assert(keys.length > 0);
  });
});

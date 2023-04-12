import assert from "assert";
import { options } from "../index";
import { ApiPromise } from "@polkadot/api";
import { MockProvider } from "@polkadot/rpc-provider/mock";
import { TypeRegistry } from "@polkadot/types";
import metadataRaw from "../metadata.json" assert { type: "json" };

describe("index", function () {
  let mock: MockProvider;
  let api: ApiPromise;

  beforeEach(async function () {
    mock = new MockProvider(new TypeRegistry());

    api = await ApiPromise.create({
      ...options,
      provider: mock,
      metadata: metadataRaw as any,
    });
  });

  afterEach(async function () {
    await api.disconnect();
    await mock.disconnect();
  });

  it("should know about runtime apis", function () {
    const topLevelRuntimeApis = Object.keys((api.registry.knownTypes as any).runtime || {});
    assert.deepEqual(topLevelRuntimeApis, [
      "AdditionalRuntimeApi",
      "HandlesRuntimeApi",
      "MessagesRuntimeApi",
      "MsaRuntimeApi",
      "SchemasRuntimeApi",
      "StatefulStorageRuntimeApi",
    ]);
  });

  it("should have rpc calls", async function () {
    assert.notEqual(api.rpc.messages, undefined);
    assert.notEqual(api.rpc.msa, undefined);
    assert.notEqual(api.rpc.schemas, undefined);
    assert.notEqual(api.rpc.frequency, undefined);
  });
});

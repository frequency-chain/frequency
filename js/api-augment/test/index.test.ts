import assert from "assert";
import { options, rpc } from "../index";
import { ApiPromise } from "@polkadot/api";
import { MockProvider } from "@polkadot/rpc-provider/mock";
import { Metadata, TypeRegistry } from "@polkadot/types";
import metadataRaw from "../metadata.json";

describe("index", function () {
  let mock: MockProvider;
  let api: ApiPromise;

  beforeEach(async function () {
    const registry = new TypeRegistry();
    registry.register(options.types);
    mock = new MockProvider(registry);

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

  it("should have rpc calls", async function () {
    assert.notEqual(api.rpc.messages, undefined);
    assert.notEqual(api.rpc.msa, undefined);
    assert.notEqual(api.rpc.schemas, undefined);
  });
});

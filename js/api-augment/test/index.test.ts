import assert from "assert";
import { options } from "../index";
import { ApiPromise } from "@polkadot/api";
import { MockProvider } from "@polkadot/rpc-provider/mock";
import { Metadata, TypeRegistry } from "@polkadot/types";
import { result as rpcMetadata } from "../metadata.json";

describe("index", function () {
  const registry = new TypeRegistry();
  registry.register(options.types);
  const metadata = new Metadata(registry, rpcMetadata as "0x{string}");
  registry.setMetadata(metadata);

  let mock: MockProvider;

  beforeEach(function (): void {
    mock = new MockProvider(new TypeRegistry());
  });

  afterEach(async function () {
    await mock.disconnect();
  });

  it("should have rpc calls", async function () {
    const api = await ApiPromise.create({
      ...options,
      provider: mock,
    });
    assert.notEqual(api.rpc.messages, undefined);
    assert.notEqual(api.rpc.msa, undefined);
    assert.notEqual(api.rpc.schemas, undefined);
    await api.disconnect();
  });
});

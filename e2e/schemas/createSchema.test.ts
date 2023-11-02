import "@frequency-chain/api-augment";

import assert from "assert";

import { AVRO_GRAPH_CHANGE } from "./fixtures/avroGraphChangeSchemaType";
import { KeyringPair } from "@polkadot/keyring/types";
import { ExtrinsicHelper } from "../scaffolding/extrinsicHelpers";
import { createKeys, createAndFundKeypair, assertExtrinsicSuccess } from "../scaffolding/helpers";
import { getFundingSource } from "../scaffolding/funding";

describe("#createSchema", function () {
  let keys: KeyringPair;
  let accountWithNoFunds: KeyringPair;
  const fundingSource = getFundingSource("schemas-create");

  before(async function () {
    keys = await createAndFundKeypair(fundingSource, 50_000_000n);
    accountWithNoFunds = createKeys();
  });

  it("should fail if account does not have enough tokens", async function () {

    await assert.rejects(ExtrinsicHelper.createSchema(accountWithNoFunds, AVRO_GRAPH_CHANGE, "AvroBinary", "OnChain").signAndSend(), {
      name: 'RpcError',
      message: /Inability to pay some fees/,
    });
  });

  it("should fail if account does not have enough tokens v2", async function () {

    await assert.rejects(ExtrinsicHelper.createSchemaV2(accountWithNoFunds, AVRO_GRAPH_CHANGE, "AvroBinary", "OnChain",[]).signAndSend(), {
      name: 'RpcError',
      message: /Inability to pay some fees/,
    });
  });

  it("should fail to create invalid schema", async function () {
    const f = ExtrinsicHelper.createSchema(keys, new Array(1000, 3), "AvroBinary", "OnChain");

    await assert.rejects(f.fundAndSend(fundingSource), {
      name: 'InvalidSchema',
    });
  });

  it("should fail to create invalid schema v2", async function () {
    const f = ExtrinsicHelper.createSchemaV2(keys, new Array(1000, 3), "AvroBinary", "OnChain", []);

    await assert.rejects(f.fundAndSend(fundingSource), {
      name: 'InvalidSchema',
    });
  });

  it("should fail to create schema less than minimum size", async function () {
    const f = ExtrinsicHelper.createSchema(keys, {}, "AvroBinary", "OnChain");
    await assert.rejects(f.fundAndSend(fundingSource), {
      name: 'LessThanMinSchemaModelBytes',
    });
  });

  it("should fail to create schema less than minimum size v2", async function () {
    const f = ExtrinsicHelper.createSchemaV2(keys, {}, "AvroBinary", "OnChain", []);
    await assert.rejects(f.fundAndSend(fundingSource), {
      name: 'LessThanMinSchemaModelBytes',
    });
  });

  it("should fail to create schema greater than maximum size", async function () {
    const maxBytes = (await ExtrinsicHelper.getSchemaMaxBytes()).toNumber();

    // Create a schema whose JSON representation is exactly 1 byte larger than the max allowed
    const hugeSchema = {
      type: "record",
      fields: [],
    }
    const hugeSize = JSON.stringify(hugeSchema).length;
    const sizeToFill = maxBytes - hugeSize - ',"name":""'.length + 1;
    hugeSchema["name"] = Array.from(Array(sizeToFill).keys()).map(i => 'a').join('');

    const f = ExtrinsicHelper.createSchema(keys, hugeSchema, "AvroBinary", "OnChain");
    await assert.rejects(f.fundAndSend(fundingSource), {
      name: 'ExceedsMaxSchemaModelBytes',
    });
  });

  it("should fail to create schema greater than maximum size v2", async function () {
    const maxBytes = (await ExtrinsicHelper.getSchemaMaxBytes()).toNumber();

    // Create a schema whose JSON representation is exactly 1 byte larger than the max allowed
    const hugeSchema = {
      type: "record",
      fields: [],
    }
    const hugeSize = JSON.stringify(hugeSchema).length;
    const sizeToFill = maxBytes - hugeSize - ',"name":""'.length + 1;
    hugeSchema["name"] = Array.from(Array(sizeToFill).keys()).map(i => 'a').join('');

    const f = ExtrinsicHelper.createSchemaV2(keys, hugeSchema, "AvroBinary", "OnChain", []);
    await assert.rejects(f.fundAndSend(fundingSource), {
      name: 'ExceedsMaxSchemaModelBytes',
    });
  });

  it("should successfully create an Avro GraphChange schema", async function () {
    const f = ExtrinsicHelper.createSchema(keys, AVRO_GRAPH_CHANGE, "AvroBinary", "OnChain");
    const { target: createSchemaEvent, eventMap } = await f.fundAndSend(fundingSource);

    assertExtrinsicSuccess(eventMap);
    assert.notEqual(createSchemaEvent, undefined);
  });

  it("should successfully create an Avro GraphChange schema v2", async function () {
    const f = ExtrinsicHelper.createSchemaV2(keys, AVRO_GRAPH_CHANGE, "AvroBinary", "OnChain", []);
    const { target: createSchemaEvent, eventMap } = await f.fundAndSend(fundingSource);

    assertExtrinsicSuccess(eventMap);
    assert.notEqual(createSchemaEvent, undefined);
  });

})

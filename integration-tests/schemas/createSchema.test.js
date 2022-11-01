const assert = require("assert");

const { BROADCAST } = require("./scaffolding/fixtures/schemaTypes");
const { AVRO, PARQUET } = require("./scaffolding/fixtures/modelTypes");
const { ON_CHAIN, IPFS } = require("./scaffolding/fixtures/payloadLocation");
const createSchema = require("./scaffolding/extrinsics/createSchema");
const fetchSchema = require("./scaffolding/query/fetchSchema");

describe("#createSchema", () => {
    it("should successfully create a BROADCAST schema", async () => {
        const {success, schemaId} = await createSchema(BROADCAST, PARQUET, IPFS);
        assert.equal(true, success);
        assert.notEqual(undefined, schemaId);

        const {key, model_type, payload_location} = await fetchSchema(schemaId);

        assert.equal(schemaId, key);
        assert.equal(PARQUET, model_type);
        assert.equal(IPFS, payload_location);
    }).timeout(1000000)
})
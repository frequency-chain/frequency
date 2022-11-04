const assert = require("assert");

const { AVRO_GRAPH_CHANGE } = require("./scaffolding/fixtures/schemaTypes");
const { AVRO, PARQUET } = require("./scaffolding/fixtures/modelTypes");
const { ON_CHAIN, IPFS } = require("./scaffolding/fixtures/payloadLocation");
const createSchema = require("./scaffolding/extrinsics/createSchema");
const fetchSchema = require("./scaffolding/query/fetchSchema");

describe("#createSchema", () => {
    it("should successfully create an Avro GraphChange schema", async () => {
        const {success, schemaId} = await createSchema(AVRO_GRAPH_CHANGE, AVRO, ON_CHAIN);
        assert.equal(true, success);
        assert.notEqual(undefined, schemaId);

        
        // To test Parquet types, we will need to patch the api-augment lib to contain Parquet types
        const {key, model_type, payload_location} = await fetchSchema(schemaId);

        assert.equal(schemaId, key);
        assert.equal(AVRO, model_type);
        assert.equal(ON_CHAIN, payload_location);
    }).timeout(1000000)
})
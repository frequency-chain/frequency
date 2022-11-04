const assert = require("assert");

const { AVRO_GRAPH_CHANGE } = require("./scaffolding/fixtures/schemaTypes");
const { AVRO, PARQUET } = require("./scaffolding/fixtures/modelTypes");
const { ON_CHAIN, IPFS } = require("./scaffolding/fixtures/payloadLocation");
const createSchema = require("./scaffolding/extrinsics/createSchema");
const fetchSchema = require("./scaffolding/query/fetchSchema");

describe("#createSchema", () => {
    it("should successfully create a BROADCAST schema", async () => {
        const {success, schemaId} = await createSchema(AVRO_GRAPH_CHANGE, AVRO, ON_CHAIN);
        assert.equal(true, success);
        assert.notEqual(undefined, schemaId);

        
        // These need a patch to the api-augment that includes a Parquet type
        const {key, model_type, payload_location} = await fetchSchema(schemaId);

        assert.equal(schemaId, key);
        assert.equal(AVRO, model_type);
        assert.equal(ON_CHAIN, payload_location);
    }).timeout(1000000)
})
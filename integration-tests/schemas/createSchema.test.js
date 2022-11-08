const assert = require("assert");

const ApiWrapper = require("./scaffolding/api/apiWrapper");
const { getFrequencyAPI, getSignerAccountKeys} = require("./scaffolding/api/apiConnection");

const { AVRO_GRAPH_CHANGE } = require("./scaffolding/fixtures/schemaTypes");
const { AVRO, PARQUET } = require("./scaffolding/fixtures/modelTypes");
const { ON_CHAIN, IPFS } = require("./scaffolding/fixtures/payloadLocation");


describe("#createSchema", () => {
    let api;

    beforeEach(async () => {
        api = new ApiWrapper(await getFrequencyAPI(), getSignerAccountKeys());
    });

    it("should successfully create an Avro GraphChange schema", async () => {
        await api.createSchema(AVRO_GRAPH_CHANGE, AVRO, ON_CHAIN);
        
        const schemaRegisteredEvent = api.getEvent("schemas.SchemaRegistered");
        const successEvent = api.getEvent("system.ExtrinsicSuccess");
        const failureEvent = api.getEvent("system.ExtrinsicFailed");

        assert.equal(true, typeof(schemaRegisteredEvent) !== "undefined" && typeof(successEvent) !== "undefined")
        assert.equal(true, typeof(failureEvent) == "undefined");

        const schemaId = schemaRegisteredEvent.data[1];
        assert.notEqual(undefined, schemaId);

        // To test Parquet types, we will need to patch the api-augment lib to contain Parquet types
        const {key, model_type, payload_location} = await api.fetchSchema(schemaId);

        assert.equal(schemaId, key);
        assert.equal(AVRO, model_type);
        assert.equal(ON_CHAIN, payload_location);
    }).timeout(1000000);
});

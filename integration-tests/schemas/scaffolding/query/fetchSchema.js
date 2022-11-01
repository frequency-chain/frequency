const { options } = require("@frequency-chain/api-augment");
const { ApiPromise, WsProvider } = require("@polkadot/api");

async function getFrequencyAPI() {
    let DEPLOY_SCHEMA_ENDPOINT_URL = process.env.DEPLOY_SCHEMA_ENDPOINT_URL;
    if (DEPLOY_SCHEMA_ENDPOINT_URL === undefined) {
      // One would think that localhost would also work here but it doesn't consistently.
      DEPLOY_SCHEMA_ENDPOINT_URL = "ws://127.0.0.1:9944";
    }
    const DefaultWsProvider = new WsProvider(DEPLOY_SCHEMA_ENDPOINT_URL);
  
    // The "options" parameter pulls in the Frequency API extrinsics
    const api = await ApiPromise.create({
      provider: DefaultWsProvider,
      ...options,
    });
    await api.isReady;
    return api;
}

const fetchSchema = async (schemaId) => {
    const api = await getFrequencyAPI();
  
    const schema = await api.rpc.schemas.getBySchemaId(schemaId);
    let schemaResult = schema.unwrap();
    const jsonSchema = Buffer.from(schemaResult.model).toString("utf8");
    const modelParsed = JSON.parse(jsonSchema);
    console.log("Schema Result", schemaResult);
    const { schema_id, model_type, payload_location } = schemaResult;
    return {
      key: schema_id.toString(),
      schema_id: schema_id.toString(),
      model_type: model_type.toString(),
      payload_location: payload_location.toString(),
      model_structure: modelParsed,
    };
};
  
module.exports = fetchSchema;
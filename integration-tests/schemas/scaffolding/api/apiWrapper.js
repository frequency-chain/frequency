module.exports = function ApiWrapper(polkadotApi, signerAccountKeys) {
    this._api = polkadotApi;
    this._keys = signerAccountKeys;

    this._eventSubscriptions = [];
    this._eventData = {};

    this._DEFAULT_SECTION = "schemas";

    this.createSchema = async (payload, modelType, payloadLocation) => {
        this._clearEventData();

        let nonce = (await this._api.rpc.system.accountNextIndex(this._keys.address)).toNumber();
        const tx = await this._api.tx.schemas.createSchema(JSON.stringify(payload), modelType, payloadLocation);
        return new Promise((resolve, reject) => {
            tx.signAndSend(signerAccountKeys, { nonce: nonce++ }, ({status, events}) => {
                console.log("Extrinsic call status:", status.type);
                if (status.isInBlock || status.isFinalized) {
                    events.forEach(({ event }) => this._storeEvent(event));
                    resolve();
                }
            })
        })
    }

    this.fetchSchema = async (schemaId) => {
        const schema = await this._api.rpc.schemas.getBySchemaId(schemaId);
        let schemaResult = schema.unwrap();
        const jsonSchema = Buffer.from(schemaResult.model).toString("utf8");
        const modelParsed = JSON.parse(jsonSchema);
        const { schema_id, model_type, payload_location } = schemaResult;

        return {
            key: schema_id.toString(),
            schema_id: schema_id.toString(),
            model_type: model_type.toString(),
            payload_location: payload_location.toString(),
            model_structure: modelParsed,
        };
    }

    // This will not work until we provision a root account
    this.setMaxSchemaSize = async (rootKeys, size) => {
        this._clearEventData();

        let nonce = (await this._api.rpc.system.accountNextIndex(rootKeys.address)).toNumber();
        const tx = await this._api.tx.schemas.setMaxSchemaModelBytes(size);
        return new Promise((resolve, reject) => {
            tx.signAndSend(signerAccountKeys, { nonce: nonce++ }, ({status, events}) => {
                console.log("Extrinsic call status:", status.type);
                if (status.isInBlock || status.isFinalized) {
                    events.forEach(({ event }) => this._storeEvent(event));
                    resolve();
                }
            })
        })
    }

    this.getEvent = (eventKey) => {
        return this._eventData[eventKey];
    }

    this._storeEvent = (polkaDotEvent) => {
        let key = `${polkaDotEvent.section}.${polkaDotEvent.method}`;
        this._eventData[key] = polkaDotEvent;
    }

    this._clearEventData = () => {
        this._eventData = {};
    }
}

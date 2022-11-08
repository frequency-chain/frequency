module.exports = function ApiWrapper(polkadotApi, signerAccountKeys) {
    this._api = polkadotApi;
    this._keys = signerAccountKeys;

    this._eventSubscriptions = [];
    this._eventData = {};

    this._DEFAULT_SECTION = "schemas";

    this.createSchema = async (payload, modelType, payloadLocation) => {
        const tx = await this._api.tx.schemas.createSchema(JSON.stringify(payload), modelType, payloadLocation);
        return this._sendTx(tx, this._keys);
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

    // NOTE:
    // The setMaxSchemaSize extrinsic checks for root privileges,
    // so this will not work until we provision a root account
    this.setMaxSchemaSize = async (rootKeys, size) => {
        const tx = await this._api.tx.schemas.setMaxSchemaModelBytes(size);
        return this._sendTx(tx, rootKeys);
    }

    this.getEvent = (eventKey) => {
        return this._eventData[eventKey];
    }

    this._sendTx = async (tx, keys) => {
        this._clearEventData();

        let nonce = (await this._api.rpc.system.accountNextIndex(keys.address)).toNumber();
        return new Promise((resolve, _reject) => {
            tx.signAndSend(keys, { nonce: nonce++ }, async ({status, events}) => {
                await this._handleTxResponse(status, events);
                resolve();
            })
        })
    }

    this._handleTxResponse = async (status, events) => {
        return new Promise((resolve, _reject) => {
            if (status.isInBlock || status.isFinalized) {
                events.forEach(({ event }) => this._storeEvent(event));
                resolve();
            }
        })
    }

    this._storeEvent = (polkaDotEvent) => {
        let key = `${polkaDotEvent.section}.${polkaDotEvent.method}`;
        this._eventData[key] = polkaDotEvent;
    }

    this._clearEventData = () => {
        this._eventData = {};
    }
}

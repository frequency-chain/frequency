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
                if (status.isFinalized) {
                    events.forEach(({ event }) => this._checkSubscriptions(event));
                    this._clearSubscriptions();
                    resolve();
                }
            })
        })
    }

    this.fetchSchema = async function(schemaId) {
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

    this.subscribeToEvent = (...eventNames) => {
        eventNames.forEach((eventName) => { this._eventSubscriptions.push(this._splitEventName(eventName)) })
        return this;
    }

    this._splitEventName = (eventName) => {
        [section, method] = eventName.split('.');
        if (method == undefined) { 
            method = section;
            section = this._DEFAULT_SECTION;
         }

         return [section, method];
    }

    this._checkSubscriptions = (polkaDotEvent) => {
        this._eventSubscriptions.forEach(([section, method]) => {
            if (polkaDotEvent.section === section && polkaDotEvent.method === method) {
                this._eventData[method] = polkaDotEvent;
            }
        })
    }

    this._clearSubscriptions = () => {
        this._eventSubscriptions = [];
    }

    this._clearEventData = () => {
        this._eventData = {};
    }
}

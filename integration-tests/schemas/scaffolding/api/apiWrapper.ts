import { SchemaResponse } from "@frequency-chain/api-augment/interfaces";
import { ApiPromise } from "@polkadot/api";
import { SubmittableExtrinsic } from "@polkadot/api/types";

interface PolkadotEvent {
    status: any,
    events: Array<any>
}

interface Wrapped<T> {
    unwrap(): T
}

interface SchemasRpc {
    getBySchemaId(schemaId: any): Wrapped<SchemaResponse>
}

export default class ApiWrapper {
    _api: ApiPromise;
    _keys: any;
    _eventData: {[eventName: string]: object}

    constructor(polkadotApi: ApiPromise, signerAccountKeys: any) {
        this._api = polkadotApi;
        this._keys = signerAccountKeys;
        this._eventData = {};
    }

    public createSchema = async (payload: any, modelType: any, payloadLocation: any): Promise<any> => {
        const tx = await this._api.tx.schemas.createSchema(JSON.stringify(payload), modelType, payloadLocation);
        return this._sendTx(tx, this._keys);
    }

    public fetchSchema = async (schemaId: any): Promise<any> => {
        // The schemas RPC is dynamically loaded into Polkadot's client.
        // Not being able to detect this is one of the things that makes TS
        // difficult to work with
        const schemasRpc: SchemasRpc = (this._api.rpc as any).schemas;
        const schema = await schemasRpc.getBySchemaId(schemaId);
        let schemaResult: SchemaResponse = schema.unwrap();
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
    public setMaxSchemaSize = async (rootKeys: any, size: any): Promise<any> => {
        const tx = await this._api.tx.schemas.setMaxSchemaModelBytes(size);
        return this._sendTx(tx, rootKeys);
    }

    public getEvent = (eventKey: string): any => {
        return this._eventData[eventKey];
    }

    private _sendTx = async (tx: any, keys: any): Promise<void> => {
        this._clearEventData();

        return new Promise((resolve, _reject) => {
            tx.signAndSend(keys, { nonce: -1 }, async ({ status, events }: PolkadotEvent) => {
                await this._handleTxResponse(status, events);
                resolve();
            });
        });
    };

    private _handleTxResponse = async (status: any, events: Array<any>): Promise<void> => {
        return new Promise((resolve, _reject) => {
            if (status.isInBlock || status.isFinalized) {
                events.forEach(({ event }) => this._storeEvent(event));
                resolve();
            }
        })
    }

    private _storeEvent = (polkaDotEvent: any): void => {
        let key = `${polkaDotEvent.section}.${polkaDotEvent.method}`;
        this._eventData[key] = polkaDotEvent;
    }

    private _clearEventData = (): void => {
        this._eventData = {};
    }
}

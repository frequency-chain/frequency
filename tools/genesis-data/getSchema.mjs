// Only way to silence PolkadotJS API warnings we don't want
console.warn = () => {
};

import { options } from '@frequency-chain/api-augment';
import {ApiPromise, WsProvider} from "@polkadot/api";
import {u8aToString} from "@polkadot/util";

/// Schema genesis format
// interface GenesisSchemaConfig {
//   schema_id: number;
//   intent_id: number;
//   model_type: string;
//   payload_location: string;
//   settings: string[];
//   model: string;
//   name: string;
// }

/// Intent genesis format
// interface GenesisIntentConfig {
//   intent_id: number;
//   name: string;
//   payload_location: string;
//   settings: string[];
// }

/// Intent group genesis format
// interface GenesisIntentGroupConfig {
//   intent_group_id: number;
//   intents: number[];
//   name: string;
// }

/// pallet-schemas genesis format
// interface GenesisSchemaPalletConfig
// {
//   /// Maximum schema identifier at genesis
//   schema_identifier_max?: number;
//   /// Maximum Intent identifier at genesis
//   intent_identifier_max?: number;
//   /// Maximum IntentGroup identifier at genesis
//   intent_group_identifier_max?: number;
//   /// Maximum schema size in bytes at genesis
//   max_schema_model_size?: number;
//   /// The list of Schemas
//   schemas?: GenesisSchema[];
//   /// The list of Intents
//   intents?: GenesisIntent[];
//   /// The list of IntentGroups
//   intent_groups?: GenesisIntentGroup[];
// }

export async function getSchemas(sourceUrl) {
    // Connect to the state source
    const sourceProvider = new WsProvider(sourceUrl);
    const sourceApi = await ApiPromise.create({provider: sourceProvider, ...options});
    const schemaSettingType = sourceApi.registry.knownTypes.types.SchemaSetting;

    const genesisSchemasList = [];
    // Schema Genesis Format
    // {
    //  "model": "{}",
    //  "model_type": "AvroBinary",
    //  "name": "dsnp.thing",
    //  "payload_location": "Itemized",
    //  "settings": ["AppendOnly", "SignatureRequired"]
    // }

    // Get all schemas
    // TODO: Switch to entriesPaged at some point
    const schemaInfos = await sourceApi.query.schemas.schemaInfos.entries();
    const genesisSchemas = schemaInfos.map(([key, value]) => {
        const v = value.toJSON();
        return {
            schema_id: key.args[0].toNumber(),
            intent_id: Number(v.intentId),
            model_type: v.modelType.toString(),
            payload_location: v.payloadLocation.toString(),
            status: v.status.toString(),
        };
    });

    const schemaModels = await sourceApi.query.schemas.schemaPayloads.entries();
    schemaModels.forEach(([key, value]) => {
        const schemaInfo = genesisSchemas.find((s) => s.schema_id === key.args[0].toNumber());
        if (!schemaInfo) {
            throw new Error(`Schema ${schemaInfo.schema_id} not found in schemaInfos.`);
        }
        schemaInfo.model = u8aToString(value.toU8a(true));
    });

    const intents = await sourceApi.query.schemas.intentInfos.entries();
    const genesisIntents = intents.map(([key, value]) => {
        const v = value.toJSON();
        const settings= new Array(16).fill(0).map((_, index) => index).filter((bit) => v.settings & (1 << bit)).map((bit) => sourceApi.registry.knownTypes.types.SchemaSetting._enum[bit]);
        return {
            intent_id: key.args[0].toNumber(),
            payload_location: v.payloadLocation.toString(),
            settings,
        };
    });

    const intentGroups = await sourceApi.query.schemas.intentGroups.entries();
    const genesisIntentGroups = intentGroups.map(([key, value]) => {
        const v = value.toJSON();
        return {
            intent_group_id: key.args[0].toNumber(),
            intents: v.intentIds.map((i) => Number(i)),
        };
    });

    const names = await sourceApi.query.schemas.nameToMappedEntityIds.entries();
    names.forEach(([key, value]) => {
        const name = `${key.args[0].toHuman()}.${key.args[1].toHuman()}`;
        const entity = value.toJSON();
        if (entity['intent']) {
            const intent = genesisIntents.find((i) => i.intent_id === Number(entity['intent']));
            if (!intent) {
                throw new Error(`Intent ${entity['intent'].toNumber()} not found in intents.`);
            }
            intent.name = name;
        } else if (entity['intentGroup']) {
            const intentGroup = genesisIntentGroups.find(({ intent_group_id }) => intent_group_id === Number(entity['intentGroup']));
            if (!intentGroup) {
                throw new Error(`IntentGroup ${entity['intentGroup'].toNumber()} not found in intentGroups.`);
            }
            intentGroup.name = name;
        }
    })

    const genesis = {
        schemas: genesisSchemas,
        intents: genesisIntents,
        intent_groups: genesisIntentGroups,
    }

    process.stdout.write(JSON.stringify(genesis, null, 2) + "\n");
}

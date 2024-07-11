// Only way to silence PolkadotJS API warnings we don't want
console.warn = () => {};

import { ApiPromise, WsProvider } from "@polkadot/api";

const options = {
  rpc: {
    schemas: {
      getBySchemaId: {
        description: "Get a Schema by Id",
        params: [
          {
            name: "schema_id",
            type: "SchemaId",
          },
        ],
        type: "Option<SchemaResponse>",
      },
    },
  },
  types: {
    SchemaId: "u16",
    SchemaModel: "Vec<u8>",
    SchemaResponse: {
      schema_id: "SchemaId",
      model: "SchemaModel",
      model_type: "ModelType",
      payload_location: "PayloadLocation",
      settings: "Vec<SchemaSetting>",
    },
    ModelType: {
      _enum: ["AvroBinary", "Parquet"],
    },
    PayloadLocation: {
      _enum: ["OnChain", "IPFS", "Itemized", "Paginated"],
    },
    SchemaSetting: {
      _enum: ["AppendOnly", "SignatureRequired"],
    },
  },
};

export async function getSchemas(sourceUrl) {
  // Connect to the state source
  const sourceProvider = new WsProvider(sourceUrl);
  const sourceApi = await ApiPromise.create({ provider: sourceProvider, ...options });

  const genesisSchemasList = [];
  // Schema Genesis Format
  // {
  //  "model": "{}",
  //  "model_type": "AvroBinary",
  //  "name": "dsnp.thing",
  //  "payload_location": "Itemized",
  //  "settings": ["AppendOnly", "SignatureRequired"]
  // }

  // Get the max schema id
  const maxSchemaId = (await sourceApi.query.schemas.currentSchemaIdentifierMaximum()).toNumber();

  // Get all schemas
  for (let id = 1; id <= maxSchemaId; id++) {
    const schemaOption = await sourceApi.rpc.schemas.getBySchemaId(id);
    if (!schemaOption.isSome) throw new Error(`Unable to get Schema Id ${id}!`);
    const schema = schemaOption.unwrap();
    genesisSchemasList[id - 1] = {
      model_type: schema.model_type.toString(),
      payload_location: schema.payload_location.toString(),
      settings: schema.settings.toJSON(),
      model: new TextDecoder().decode(schema.model),
      name: "",
    };
  }

  // Get all the schema names TODO: Switch to entriesPaged at some point
  const names = await sourceApi.query.schemas.schemaNameToIds.entries();

  for (const storageName of names) {
    const name = storageName[0].toHuman();
    for (const idBig of storageName[1].ids) {
      const id = idBig.toNumber();
      if (genesisSchemasList[id - 1]) {
        genesisSchemasList[id - 1].name = name.join(".");
      }
    }
  }

  process.stdout.write(JSON.stringify(genesisSchemasList, null, 2) + "\n");
}

// Only way to silence PolkadotJS API warnings we don't want
console.warn = () => {};
import { ApiPromise, WsProvider } from '@polkadot/api';
import { promises as fs } from 'fs';

const options = {
  rpc: {
    schemas: {
      getBySchemaId: {
        description: 'Get a Schema by Id',
        params: [
          {
            name: 'schema_id',
            type: 'SchemaId',
          },
        ],
        type: 'Option<SchemaResponse>',
      },
      getVersions: {
        description: 'Get different versions and schema ids for a complete schema name or only a namespace',
        params: [
          {
            name: 'schema_name',
            type: 'String',
          },
        ],
        type: 'Option<Vec<SchemaVersionResponse>>',
      },
    },
  },
  types: {
    SchemaId: 'u16',
    SchemaModel: 'Vec<u8>',
    SchemaVersion: 'u8',
    SchemaResponse: {
      schema_id: 'SchemaId',
      model: 'SchemaModel',
      model_type: 'ModelType',
      payload_location: 'PayloadLocation',
      settings: 'Vec<SchemaSetting>',
    },
    ModelType: {
      _enum: ['AvroBinary', 'Parquet'],
    },
    PayloadLocation: {
      _enum: ['OnChain', 'IPFS', 'Itemized', 'Paginated'],
    },
    SchemaSetting: {
      _enum: ['AppendOnly', 'SignatureRequired'],
    },
    SchemaVersionResponse: {
      schema_name: 'String',
      schema_version: 'SchemaVersion',
      schema_id: 'SchemaId',
    },
  },
};
const SOURCE_URL = 'wss://1.rpc.frequency.xyz';
const GENERATED_FILE_NAME = '../data.ts';

async function main() {
  try {
    await getSchemas(SOURCE_URL);
    process.exit(0);
  } catch (error) {
    console.error('Error:', error);
    process.exit(1);
  }
}

main();

async function getSchemas(sourceUrl) {
  const provider = new WsProvider(sourceUrl);
  const api = await ApiPromise.create({ provider, ...options });
  const nameToIds = await api.rpc.schemas.getVersions('dsnp');

  const idToSchemaInfo = new Map();
  const fullNameToVersions = new Map();

  for (const v of JSON.parse(JSON.stringify(nameToIds))) {
    const fullName = v.schema_name;
    const names = fullName.split('.');
    const version = v.schema_version;
    const id = v.schema_id;

    if (fullNameToVersions.has(fullName)) {
      const arr = fullNameToVersions.get(fullName);
      arr.push({
        version: version,
        id: id,
      });
      fullNameToVersions.set(fullName, arr);
    } else {
      fullNameToVersions.set(fullName, [
        {
          version: version,
          id: id,
        },
      ]);
    }

    const chainSchemaInfo = await api.query.schemas.schemaInfos(id);
    const objSchemaInfo = JSON.parse(JSON.stringify(chainSchemaInfo));
    idToSchemaInfo.set(id, {
      id: id,
      namespace: names[0],
      name: names[1],
      version: version,
      deprecated: false,
      modelType: objSchemaInfo.modelType,
      payloadLocation: objSchemaInfo.payloadLocation,
      appendOnly: (objSchemaInfo.settings & 1) > 0,
      signatureRequired: (objSchemaInfo.settings & 2) > 0,
    });
  }

  for (let arr of fullNameToVersions.values()) {
    arr.sort((a, b) => a.version - b.version);
    for (let i = 0; i < arr.length; i++) {
      const inner = arr[i];
      const info = idToSchemaInfo.get(inner.id);
      info.deprecated = i < arr.length - 1;
    }
  }

  let generated = '';
  for (let v of idToSchemaInfo.values()) {
    generated += `\n${JSON.stringify(v, null, 2).replace(/"([^"]+)":/g, '$1:')},`;
  }
  let output = `import { SchemaInfo } from './schemas';\n\nexport const SCHEMA_INFOS: SchemaInfo[] = [${generated}\n];\n`;

  console.log(output);
  await fs.writeFile(GENERATED_FILE_NAME, output, 'utf8');
}

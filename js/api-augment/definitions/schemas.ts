export default {
  rpc: {
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
    checkSchemaValidity: {
      description: '',
      params: [
        {
          name: 'model',
          type: 'SchemaModel',
        },
        {
          name: 'at',
          type: 'BlockHash',
          isOptional: true,
        },
      ],
      type: 'bool',
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

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
    IntentGroupId: 'u16',
    IntentGroupResponse: {
      intent_group_id: 'IntentGroupId',
      intent_ids: 'Vec<IntentId>',
    },
    IntentId: 'u16',
    IntentResponse: {
      intent_id: 'IntentId',
      payload_location: 'PayloadLocation',
      settings: 'Vec<IntentSetting>',
      schema_ids: 'Option<Vec<SchemaId>>',
    },
    IntentSetting: {
      _enum: ['AppendOnly', 'SignatureRequired'],
    },
    MappedEntityIdentifier: {
      _enum: {
        Intent: 'IntentId',
        IntentGroup: 'IntentGroupId',
      },
    },
    ModelType: {
      _enum: ['AvroBinary', 'Parquet'],
    },
    NameLookupResponse: {
      name: 'String',
      entity_id: 'MappedEntityIdentifier',
    },
    PayloadLocation: {
      _enum: ['OnChain', 'IPFS', 'Itemized', 'Paginated'],
    },
    SchemaInfoResponse: {
      schema_id: 'SchemaId',
      intent_id: 'IntentId',
      model_type: 'ModelType',
      status: 'SchemaStatus',
      payload_location: 'PayloadLocation',
      settings: 'Vec<IntentSetting>',
    },
    SchemaResponseV2: {
      schema_id: 'SchemaId',
      intent_id: 'IntentId',
      model: 'SchemaModel',
      model_type: 'ModelType',
      payload_location: 'PayloadLocation',
      settings: 'Vec<IntentSetting>',
      status: 'SchemaStatus',
    },
    SchemaSetting: {
      _enum: ['AppendOnly', 'SignatureRequired'],
    },
    SchemaStatus: {
      _enum: ['Active', 'Deprecated', 'Unsupported'],
    },
    SchemaVersionResponse: {
      schema_name: 'String',
      schema_version: 'SchemaVersion',
      schema_id: 'SchemaId',
    },
  },
};

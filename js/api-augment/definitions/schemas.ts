export default {
  rpc: {},
  types: {
    IntentGroupId: 'u16',
    IntentGroupResponse: {
      intent_group_id: 'IntentGroupId',
      intent_ids: 'Vec<IntentId>',
    },
    IntentId: 'u16',
    IntentResponse: {
      intent_id: 'IntentId',
      payload_location:  'PayloadLocation',
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
      }
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
    SchemaId: 'u16',
    SchemaInfoResponse: {
      schema_id: 'SchemaId',
      intent_id: 'IntentId',
      model_type: 'ModelType',
      status: 'SchemaStatus',
      payload_location: 'PayloadLocation',
      settings: 'Vec<IntentSetting>',
    },
    SchemaResponse: {
      schema_id: 'SchemaId',
      model: 'String',
      model_type: 'ModelType',
      payload_location: 'PayloadLocation',
      settings: 'Vec<SchemaSetting>',
    },
    SchemaResponseV2: {
      schema_id: 'SchemaId',
      intent_id: 'IntentId',
      model: 'String',
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
    SchemaVersion: 'u8',
    SchemaVersionResponse: {
      schema_name: 'String',
      schema_version: 'SchemaVersion',
      schema_id: 'SchemaId',
    },
  },
};

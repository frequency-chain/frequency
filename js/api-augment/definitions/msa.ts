export default {
  rpc: {
    checkDelegations: {
      description: 'Test a list of MSAs to see if they have delegated to the provider MSA',
      params: [
        {
          name: 'delegator_msa_ids',
          type: 'Vec<DelegatorId>',
        },
        {
          name: 'provider_msa_id',
          type: 'ProviderId',
        },
        {
          name: 'block_number',
          type: 'BlockNumber',
        },
        {
          name: 'schema_id',
          type: 'Option<SchemaId>',
        },
      ],
      type: 'Vec<(DelegatorId, bool)>',
    },
    grantedSchemaIdsByMsaId: {
      description: 'Fetch the list of schema ids that a delegator has granted to provider',
      params: [
        {
          name: 'delegator_msa_id',
          type: 'DelegatorId',
        },
        {
          name: 'provider_msa_id',
          type: 'ProviderId',
        },
      ],
      type: 'Option<Vec<SchemaGrantResponse>>',
    },
    getKeysByMsaId: {
      description: 'Fetch Keys for an MSA Id',
      params: [
        {
          name: 'msa_id',
          type: 'MessageSourceId',
        },
      ],
      type: 'Option<KeyInfoResponse>',
    },
  },
  types: {
    MessageSourceId: 'u64',
    DelegatorId: 'MessageSourceId',
    ProviderId: 'MessageSourceId',
    KeyInfoResponse: {
      msa_keys: 'Vec<AccountId>',
      msa_id: 'MessageSourceId',
    },
    SchemaGrantResponse: {
      schema_id: 'SchemaId',
      revoked_at: 'BlockNumber',
    },
  },
  runtime: {
    MsaRuntimeApi: [
      {
        methods: {
          has_delegation: {
            description:
              'Check to see if a delegation existed between the given delegator and provider at a given block',
            params: [
              {
                name: 'delegator_id',
                type: 'DelegatorId',
              },
              {
                name: 'provider_id',
                type: 'ProviderId',
              },
              {
                name: 'block_number',
                type: 'BlockNumber',
              },
              {
                name: 'schema_id',
                type: 'Option<SchemaId>',
              },
            ],
            type: 'bool',
          },
          get_granted_schemas_by_msa_id: {
            description:
              'Get the list of schema ids (if any) that exist in any delegation between the delegator and provider',
            params: [
              {
                name: 'delegator_id',
                type: 'DelegatorId',
              },
              {
                name: 'provider_id',
                type: 'ProviderId',
              },
            ],
            type: 'Option<Vec<SchemaId>>',
          },
        },
        version: 1,
      },
      {
        methods: {
          has_delegation: {
            description:
              'Check to see if a delegation existed between the given delegator and provider at a given block',
            params: [
              {
                name: 'delegator_id',
                type: 'DelegatorId',
              },
              {
                name: 'provider_id',
                type: 'ProviderId',
              },
              {
                name: 'block_number',
                type: 'BlockNumber',
              },
              {
                name: 'schema_id',
                type: 'Option<SchemaId>',
              },
            ],
            type: 'bool',
          },
          get_granted_schemas_by_msa_id: {
            description:
              'Get the list of schema ids (if any) that exist in any delegation between the delegator and provider',
            params: [
              {
                name: 'delegator_id',
                type: 'DelegatorId',
              },
              {
                name: 'provider_id',
                type: 'ProviderId',
              },
            ],
            type: 'Option<Vec<SchemaGrantResponse>>',
          },
        },
        version: 2,
      },
    ],
  },
};

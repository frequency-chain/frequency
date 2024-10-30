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
    getAllGrantedDelegationsByMsaId: {
      description: 'Get the list of all delegated providers with schema permission grants',
      params: [
        {
          name: 'delegator_msa_id',
          type: 'DelegatorId',
        },
      ],
      type: 'Vec<DelegationResponse>',
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
    DelegationResponse: {
      provider_id: 'ProviderId',
      permissions: 'Vec<SchemaGrantResponse>',
    },
    // Runtime types
    // Not sure why these have to be noted here, but they do
    CommonPrimitivesMsaDelegatorId: 'u64',
    CommonPrimitivesMsaProviderId: 'u64',
  },
};

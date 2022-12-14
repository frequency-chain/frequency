export default {
  rpc: {
    // // *Temporarily Removed* until https://github.com/LibertyDSNP/frequency/issues/418
    // getMsaKeys: {
    //   description: "Fetch Keys for an MSA Id",
    //   params: [
    //     {
    //       name: "msa_id",
    //       type: "MessageSourceId",
    //     },
    //   ],
    //   type: "Vec<KeyInfoResponse>",
    // },
    checkDelegations: {
      description: "Test a list of MSAs to see if they have delegated to the provider MSA",
      params: [
        {
          name: "delegator_msa_ids",
          type: "Vec<DelegatorId>",
        },
        {
          name: "provider_msa_id",
          type: "ProviderId",
        },
        {
          name: "block_number",
          type: "BlockNumber",
        },
        {
          name: "schema_id",
          type: "Option<SchemaId>",
        },
      ],
      type: "Vec<(DelegatorId, bool)>",
    },
    grantedSchemaIdsByMsaId: {
      description: "Fetch the list of schema ids that a delegator has granted to provider",
      params: [
        {
          name: "delegator_msa_id",
          type: "DelegatorId",
        },
        {
          name: "provider_msa_id",
          type: "ProviderId",
        },
      ],
      type: "Option<Vec<SchemaId>>",
    },
    didToMsaId: {
      description: "Given a DID, retrieve the MSA Id, if it is registered and active.",
      params: [
        {name: "did", type: "Vec<u8>"},
      ],
      type: "Option<MessageSourceId>",
    },
    resolveDid: {
      description: "Convert a given MSA Id to a DID document",
      params: [
        {name: "did", type: "Vec<u8>"},
      ],
      type: "Option<String>",
    }
  },
  types: {
    MessageSourceId: "u64",
    DelegatorId: "MessageSourceId",
    ProviderId: "MessageSourceId",
    KeyInfoResponse: {
      key: "AccountId",
      msaId: "MessageSourceId",
    },
  },
  runtime: {
    MsaRuntimeApi: [
      {
        methods: {
          has_delegation: {
            description:
              "Check to see if a delegation existed between the given delegator and provider at a given block",
            params: [
              {
                name: "delegator_id",
                type: "DelegatorId",
              },
              {
                name: "provider_id",
                type: "ProviderId",
              },
              {
                name: "block_number",
                type: "BlockNumber",
              },
              {
                name: "schema_id",
                type: "Option<SchemaId>",
              },
            ],
            type: "bool",
          },
          get_granted_schemas_by_msa_id: {
            description:
              "Get the list of schema ids (if any) that exist in any delegation between the delegator and provider",
            params: [
              {
                name: "delegator_id",
                type: "DelegatorId",
              },
              {
                name: "provider_id",
                type: "ProviderId",
              },
            ],
            type: "Option<Vec<SchemaId>>",
          },
          get_public_key_count_by_msa_id: {
            description: "Get the number of keys associated with an MSA Id",
            params: [
              {
                name: "msa_id",
                type: "MessageSourceId",
              },
            ],
            type: "u8",
          },
          get_providers_for_msa_id: {
            description: "Get a list of provider MSA Ids for a given MSA Id",
            params: [
              {
                name: "msa_id",
                type: "MessageSourceId",
              },
            ],
            type: "Vec<ProviderId>",
          },
        },
        version: 1,
      },
    ],
  },
};

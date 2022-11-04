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
          type: "Vec<MessageSourceId>",
        },
        {
          name: "provider_msa_id",
          type: "MessageSourceId",
        },
        {
          name: "block_number",
          type: "Option<BlockNumber>",
        },
      ],
      type: "Vec<(MessageSourceId, bool)>",
    },
    grantedSchemaIdsByMsaId: {
      description: "Fetch the list of schema ids that a delegator has granted to provider",
      params: [
        {
          name: "delegator_msa_id",
          type: "MessageSourceId",
        },
        {
          name: "provider_msa_id",
          type: "MessageSourceId",
        },
      ],
      type: "Option<Vec<SchemaId>>",
    },
  },
  types: {
    MessageSourceId: "u64",
    Delegator: "MessageSourceId",
    Provider: "MessageSourceId",
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
          description: 'Check to see if a delegation existed between the given delegator and provider at a given block',
          params: [
            {
              name: "delegator",
              type: "Delegator"
            },
            {
              name: "provider",
              type: "Provider"
            },
            {
              name: "block_number",
              type: "Option<BlockNumber>"
            }
          ],
          type: 'bool'
        },
        get_granted_schemas_by_msa_id: {
          description: 'Get the list of schema ids (if any) that exist in any delegation between the delegator and provider',
          params: [
            {
              name: "delegator",
              type: "Delegator"
            },
            {
              name: "provider",
              type: "Provider"
            },
          ],
          type: 'Option<Vec<SchemaId>>'
        }
      },
      version: 1
    }
  ]}
};

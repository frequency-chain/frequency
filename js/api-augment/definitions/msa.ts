export default {
  rpc: {
    getMsaId: {
      description: "Fetch MSA Id by Key",
      params: [
        {
          name: "key",
          type: "AccountId",
        },
      ],
      type: "Option<MessageSourceId>",
    },
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
        }
      ],
      type: "Vec<(MessageSourceId, bool)>",
    },
    grantedSchemaIds: {
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
      type: "Vec<SchemaId>",
    },
  },
  types: {
    MessageSourceId: "u64",
    KeyInfoResponse: {
      key: "AccountId",
      msaId: "MessageSourceId",
    },
  },
};

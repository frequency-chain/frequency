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
    getMsaKeys: {
      description: "Fetch Keys for an MSA Id",
      params: [
        {
          name: "msa_id",
          type: "MessageSourceId",
        },
      ],
      type: "Vec<KeyInfoResponse>",
    },
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
      ],
      type: "Vec<(MessageSourceId, bool)>",
    },
  },
  types: {
    MessageSourceId: "u64",
    KeyInfoResponse: {
      key: "AccountId",
      msaId: "MessageSourceId",
      nonce: "u32",
      expired: "BlockNumber",
    },
  },
};

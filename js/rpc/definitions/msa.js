export default {
  rpc: {
    msa: {
      getMsaId: {
        description: "Fetch MSA Id by Key",
        params: [
          {
            name: "key",
            type: "AccountId",
          },
        ],
        type: "Option<MessageSenderId>",
      },
      getMsaKeys: {
        description: "Fetch Keys for an MSA Id",
        params: [
          {
            name: "msa_id",
            type: "MessageSenderId",
          },
        ],
        type: "Vec<KeyInfoResponse>",
      },
      checkDelegations: {
        description: "Test a list of MSAs to see if they have delegated to the provider MSA",
        params: [
          {
            name: "delegator_msa_ids",
            type: "Vec<MessageSenderId>",
          },
          {
            name: "provider_msa_id",
            type: "MessageSenderId",
          },
        ],
        type: "Vec<(MessageSenderId, bool)>",
      },
    },
  },
  types: {
    MessageSenderId: "u64",
    KeyInfoResponse: {
      key: "AccountId",
      msaId: "MessageSenderId",
      nonce: "u32",
      expired: "BlockNumber",
    },
  },
};

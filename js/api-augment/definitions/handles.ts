export default {
  rpc: {
    getHandleForMsa: {
      description: 'Get handle for a given msa_id',
      params: [
        {
          name: 'msa_id',
          type: 'MessageSourceId',
        },
      ],
      type: 'Option<HandleResponse>',
    },
    getMsaForHandle: {
      description: 'Get msa_id for a given handle',
      params: [
        {
          name: 'display_handle',
          type: 'String',
        },
      ],
      type: 'Option<MessageSourceId>',
    },
    getNextSuffixes: {
      description: 'Get next suffixes for a given handle and count',
      params: [
        {
          name: 'base_handle',
          type: 'String',
        },
        {
          name: 'count',
          type: 'u16',
        },
      ],
      type: 'PresumptiveSuffixesResponse',
    },
    validateHandle: {
      description: 'Check whether the supplied handle passes all the checks performed by claim_handle call.',
      params: [
        {
          name: 'base_handle',
          type: 'String',
        },
      ],
      type: 'bool',
    },
  },
  types: {
    HandleSuffix: 'u16',
    HandleResponse: {
      base_handle: 'String',
      canonical_base: 'String',
      suffix: 'u16',
    },
    PresumptiveSuffixesResponse: {
      suffixes: 'Vec<HandleSuffix>',
      base_handle: 'String',
    },
  },
};

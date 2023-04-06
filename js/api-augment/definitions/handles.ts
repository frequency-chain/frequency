export default {
  rpc: {
    getHandleForMsa: {
      description: "Get handle for a given msa_id",
      params: [
        {
          name: "msa_id",
          type: "MessageSourceId",
        },
      ],
      type: "Option<HandleResponse>",
    },
    getNextSuffixes: {
      description: "Get next suffixes for a given handle and count",
      params: [
        {
          name: "handle_input",
          type: "PresumtiveSuffixesRequest",
        },
      ],
      type: "PresumtiveSuffixesResponse",
    },
  },
  types: {
    HandleSuffix: "u16",
    HandleResponse: {
      base_handle: "Vec<u8>",
      canonical_handle: "Vec<u8>",
      suffix: "u16",
    },
    PresumtiveSuffixesRequest: {
      base_handle: "Vec<u8>",
      count: "u16",
    },
    PresumtiveSuffixesResponse: {
      suffixes: "Vec<HandleSuffix>",
      base_handle: "Vec<u8>",
    },
  },
  runtime: {
    HandlesRuntimeApi: [
      {
        methods: {
          get_handle_for_msa: {
            description: "Get handle for a given msa_id",
            params: [
              {
                name: "msa_id",
                type: "MessageSourceId",
              },
            ],
            type: "Option<HandleResponse>",
          },
          get_next_suffixes: {
            description: "Get next suffixes for a given handle and count",
            params: [
              {
                name: "handle_input",
                type: "PresumtiveSuffixesRequest",
              },
            ],
            type: "PresumtiveSuffixesResponse",
          },
        },
        version: 1,
      },
    ],
  },
};

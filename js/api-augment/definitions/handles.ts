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
          type: "PresumptiveSuffixesRequest",
        },
      ],
      type: "PresumptiveSuffixesResponse",
    },
  },
  types: {
    HandleSuffix: "u16",
    HandleResponse: {
      base_handle: "String",
      canonical_handle: "String",
      suffix: "u16",
    },
    PresumptiveSuffixesRequest: {
      base_handle: "String",
      count: "u16",
    },
    PresumptiveSuffixesResponse: {
      suffixes: "Vec<HandleSuffix>",
      base_handle: "String",
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
                type: "PresumptiveSuffixesRequest",
              },
            ],
            type: "PresumptiveSuffixesResponse",
          },
        },
        version: 1,
      },
    ],
  },
};

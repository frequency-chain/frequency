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
          name: "handle",
          type: "Vec<u8>",
        },
        {
          name: "count",
          type: "u16",
        },
      ],
      type: "Vec<u16>",
    },
  },
  types: {
    HandleResponse: {
      base_handle: "Vec<u8>",
      canonical_handle: "Vec<u8>",
      suffix: "u16",
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
                name: "handle",
                type: "Vec<u8>",
              },
              {
                name: "count",
                type: "u16",
              },
            ],
            type: "Vec<u16>",
          },
        },
        version: 1,
      },
    ],
  },
};

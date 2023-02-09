export default {
  rpc: {
    getPages: {
      description: "Gets pages of stateful storage",
      params: [
        {
          name: "msa_id",
          type: "MessageSourceId",
        },
        {
          name: "schema_id",
          type: "SchemaId",
        },
      ],
      type: "Vec<StatefulStorageResponse>",
    },
  },
  types: {
    PageId: "u16",
    StatefulStorageResponse: {
      index_id: "PageId",
      msa_id: "MessageSourceId",
      schema_id: "SchemaId",
      payload: "Vec<u8>",
    },
  },
  runtime: {
    StatefulStorageRuntimeApi: [
      {
        methods: {
          get_pages: {
            description: "Fetch the stateful storage pages by msa_id and schema_id",
            params: [
              {
                name: "msa_id",
                type: "MessageSourceId",
              },
              {
                name: "schema_id",
                type: "SchemaId",
              },
            ],
            type: "Vec<StatefulStorageResponse>",
          },
        },
        version: 1,
      },
    ],
  },
};

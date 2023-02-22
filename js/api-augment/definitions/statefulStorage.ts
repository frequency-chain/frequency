export default {
  rpc: {
    getPaginatedStorages: {
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
      type: "Vec<PaginatedStorageResponse>",
    },
    getItemizedStorages: {
      description: "Gets itemized of stateful storages",
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
      type: "ItemizedStoragePageResponse",
    },
  },
  types: {
    PageId: "u16",
    PageHash: "u32",
    ItemizedStorageResponse: {
      index: "u16",
      payload: "Vec<u8>",
    },
    PaginatedStorageResponse: {
      page_id: "PageId",
      msa_id: "MessageSourceId",
      schema_id: "SchemaId",
      content_hash: "PageHash",
      payload: "Vec<u8>",
    },
    ItemizedStoragePageResponse: {
      msa_id: "MessageSourceId",
      schema_id: "SchemaId",
      content_hash: "PageHash",
      items: "Vec<ItemizedStorageResponse>",
    },
  },
  runtime: {
    StatefulStorageRuntimeApi: [
      {
        methods: {
          get_paginated_storages: {
            description: "Fetch the stateful paginated storages by msa_id and schema_id",
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
            type: "Result<Vec<PaginatedStorageResponse>, SpRuntimeDispatchError>",
          },
          get_itemized_storages: {
            description: "Fetch the stateful itemized storages by msa_id and schema_id",
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
            type: "Result<ItemizedStoragePageResponse, SpRuntimeDispatchError>",
          },
        },
        version: 1,
      },
    ],
  },
};

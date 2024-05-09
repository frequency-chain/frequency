export default {
  rpc: {
    getPaginatedStorage: {
      description: 'Gets pages of stateful storage',
      params: [
        {
          name: 'msa_id',
          type: 'MessageSourceId',
        },
        {
          name: 'schema_id',
          type: 'SchemaId',
        },
      ],
      type: 'Vec<PaginatedStorageResponse>',
    },
    getItemizedStorage: {
      description: 'Gets itemized of stateful storage',
      params: [
        {
          name: 'msa_id',
          type: 'MessageSourceId',
        },
        {
          name: 'schema_id',
          type: 'SchemaId',
        },
      ],
      type: 'ItemizedStoragePageResponse',
    },
  },
  types: {
    PageId: 'u16',
    PageHash: 'u32',
    PageNonce: 'u16',
    ItemizedStorageResponse: {
      index: 'u16',
      payload: 'Vec<u8>',
    },
    PaginatedStorageResponse: {
      page_id: 'PageId',
      msa_id: 'MessageSourceId',
      schema_id: 'SchemaId',
      content_hash: 'PageHash',
      nonce: 'PageNonce',
      payload: 'Vec<u8>',
    },
    ItemizedStoragePageResponse: {
      msa_id: 'MessageSourceId',
      schema_id: 'SchemaId',
      content_hash: 'PageHash',
      nonce: 'PageNonce',
      items: 'Vec<ItemizedStorageResponse>',
    },
  },
  runtime: {
    StatefulStorageRuntimeApi: [
      {
        methods: {
          get_paginated_storage: {
            description: 'Fetch the stateful paginated storage by msa_id and schema_id',
            params: [
              {
                name: 'msa_id',
                type: 'MessageSourceId',
              },
              {
                name: 'schema_id',
                type: 'SchemaId',
              },
            ],
            type: 'Result<Vec<PaginatedStorageResponse>, SpRuntimeDispatchError>',
          },
          get_itemized_storage: {
            description: 'Fetch the stateful itemized storage by msa_id and schema_id',
            params: [
              {
                name: 'msa_id',
                type: 'MessageSourceId',
              },
              {
                name: 'schema_id',
                type: 'SchemaId',
              },
            ],
            type: 'Result<ItemizedStoragePageResponse, SpRuntimeDispatchError>',
          },
        },
        version: 1,
      },
    ],
  },
};

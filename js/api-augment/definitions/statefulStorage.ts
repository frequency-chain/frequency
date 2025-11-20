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
          name: 'intent_id',
          type: 'IntentId',
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
          name: 'intent_id',
          type: 'IntentId',
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
      schema_id: 'SchemaId',
      payload: 'Vec<u8>',
    },
    PaginatedStorageResponse: {
      page_id: 'PageId',
      msa_id: 'MessageSourceId',
      intent_id: 'IntentId',
      schema_id: 'SchemaId',
      content_hash: 'PageHash',
      nonce: 'PageNonce',
      payload: 'Vec<u8>',
    },
    ItemizedStoragePageResponse: {
      msa_id: 'MessageSourceId',
      intent_id: 'IntentId',
      content_hash: 'PageHash',
      nonce: 'PageNonce',
      items: 'Vec<ItemizedStorageResponse>',
    },
  },
};

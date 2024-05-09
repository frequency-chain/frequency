export default {
  rpc: {
    getBySchemaId: {
      description: 'Get messages by schemaId paginated',
      params: [
        {
          name: 'schema_id',
          type: 'SchemaId',
        },
        {
          name: 'pagination',
          type: 'BlockPaginationRequest',
        },
      ],
      type: 'BlockPaginationResponseMessage',
    },
  },
  types: {
    BlockPaginationRequest: {
      from_block: 'BlockNumber', // inclusive
      from_index: 'u32', // starts from 0
      to_block: 'BlockNumber', // exclusive
      page_size: 'u32',
    },
    MessageResponse: {
      payload: 'Option<Vec<u8>>', //  Serialized data in a user-defined schema format
      cid: 'Option<Vec<u8>>', // The content address for an IPFS payload
      provider_msa_id: 'MessageSourceId', //  Message source account id of the Provider
      msa_id: 'Option<MessageSourceId>', //  Message source account id (the original source)
      index: 'u16', // index in block to get total order
      block_number: 'BlockNumber',
      payload_length: 'Option<u32>', // Length of IPFS payload file
    },
    BlockPaginationResponseMessage: {
      content: 'Vec<MessageResponse>',
      has_next: 'bool',
      next_block: 'Option<BlockNumber>',
      next_index: 'Option<u32>',
    },
  },
  runtime: {
    MessagesRuntimeApi: [
      {
        methods: {
          get_messages_by_schema_and_block: {
            description: 'Retrieve the messages for a particular schema and block number',
            params: [
              {
                name: 'schema_id',
                type: 'SchemaId',
              },
              {
                name: 'schema_payload_location',
                type: 'PayloadLocation',
              },
              {
                name: 'block_number',
                type: 'BlockNumber',
              },
            ],
            type: 'Vec<MessageResponse>',
          },
          get_schema_by_id: {
            description: 'Retrieve a schema by id',
            params: [
              {
                name: 'schema_id',
                type: 'SchemaId',
              },
            ],
            type: 'Option<SchemaResponse>',
          },
        },
        version: 1,
      },
    ],
  },
};

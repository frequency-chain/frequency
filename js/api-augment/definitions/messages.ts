export default {
  rpc: {
    getBySchema: {
      description: "Get messages by schemaId paginated",
      params: [
        {
          name: "schema_id",
          type: "SchemaId",
        },
        {
          name: "pagination",
          type: "BlockPaginationRequest",
        },
      ],
      type: "BlockPaginationResponseMessage",
    },
  },
  types: {
    BlockPaginationRequest: {
      from_block: "BlockNumber", // inclusive
      from_index: "u32", // starts from 0
      to_block: "BlockNumber", // exclusive
      page_size: "u32",
    },
    MessageResponse: {
      payload: "Vec<u8>", //  Serialized data in a user-defined schema format
      provider_msa_id: "MessageSourceId", //  Message source account id of the Provider
      msa_id: "MessageSourceId", //  Message source account id (the original source)
      index: "u16", // index in block to get total order
      block_number: "BlockNumber",
      payload_length: "u32",
    },
    BlockPaginationResponseMessage: {
      content: "Vec<MessageResponse>",
      has_next: "bool",
      next_block: "Option<BlockNumber>",
      next_index: "Option<u32>",
    },
  },
};

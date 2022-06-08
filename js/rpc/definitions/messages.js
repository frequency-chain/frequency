export default {
  rpc: {
    messages: {
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
  },
  types: {
    BlockPaginationRequest: {
      from_block: "BlockNumber", // inclusive
      from_index: "u32", // starts from 0
      to_block: "BlockNumber", // exclusive
      page_size: "u32",
    },
    MessageResponse: {
      data: "Vec<u8>", //  Serialized data in a user-defined schema format
      signer: "AccountId", //  Signature of the signer
      msa_id: "MessageSenderId", //  Message source account id (the original sender)
      index: "u16", // index in block to get total order
      block_number: "BlockNumber",
    },
    BlockPaginationResponseMessage: {
      content: "Vec<MessageResponse>",
      has_next: "bool",
      next_block: "Option<BlockNumber>",
      next_index: "Option<u32>",
    },
  },
};

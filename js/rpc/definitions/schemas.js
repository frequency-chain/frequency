export default {
  rpc: {
    schemas: {
      getLatestSchemaId: {
        description:
          "Get the most recent (aka highest) Schema Id. Useful for then retrieving a list of all Schemas (1-[result])",
        params: [
          {
            name: "at",
            type: "BlockHash",
            optional: true,
          },
        ],
        type: "SchemaId",
      },
      getBySchemaId: {
        description: "Get a Schema by Id",
        params: [
          {
            name: "schema_id",
            type: "SchemaId",
          },
        ],
        type: "Option<SchemaResponse>",
      },
      checkSchemaValidity: {
        description: "",
        params: [
          {
            name: "at",
            type: "BlockHash",
            optional: true,
          },
          {
            name: "format",
            type: "SchemaFormat",
          },
        ],
        type: "bool",
      },
    },
  },
  types: {
    SchemaId: "u16",
    SchemaFormat: "Vec<u8>",
    SchemaResponse: {
      schema_id: "SchemaId",
      format: "SchemaFormat",
    },
  },
};

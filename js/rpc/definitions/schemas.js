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
            name: "model",
            type: "SchemaModel",
          },
        ],
        type: "bool",
      },
    },
  },
  types: {
    SchemaId: "u16",
    SchemaModel: "Vec<u8>",
    ModelType: {
      _enum: ["AvroBinary"],
    },
    SchemaResponse: {
      schema_id: "SchemaId",
      model: "SchemaModel",
      model_type: "ModelType",
    },
  },
};

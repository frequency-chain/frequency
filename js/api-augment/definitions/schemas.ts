export default {
  rpc: {
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
          name: "model",
          type: "SchemaModel",
        },
        {
          name: "at",
          type: "BlockHash",
          isOptional: true,
        },
      ],
      type: "bool",
    },
  },
  types: {
    SchemaId: "u16",
    SchemaModel: "Vec<u8>",
    SchemaResponse: {
      schema_id: "SchemaId",
      model: "SchemaModel",
      model_type: "ModelType",
      payload_location: "PayloadLocation",
      settings: "Vec<SchemaSetting>",
    },
    ModelType: {
      _enum: ["AvroBinary", "Parquet"],
    },
    PayloadLocation: {
      _enum: ["OnChain", "IPFS", "Itemized", "Paginated"],
    },
    SchemaSetting: {
      _enum: ["AppendOnly", "SignatureRequired"],
    },
  },
  runtime: {
    SchemasRuntimeApi: [
      {
        methods: {
          get_schema_by_id: {
            description: "Fetch the schema by id",
            params: [
              {
                name: "schema_id",
                type: "SchemaId",
              },
            ],
            type: "Option<SchemaResponse>",
          },
        },
        version: 1,
      },
    ],
  },
};

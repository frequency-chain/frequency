export default {
  rpc: {
    computeExtrinsicCost: {
      description: "Retrieve the Fee Details for a given encoded transaction",
      params: [
        {
          name: "encoded_xt",
          type: "Bytes",
        },
        {
          name: "at",
          type: "BlockHash",
          optional: true,
        },
      ],
      type: "FeeDetails",
    },
  },
  types: {},
};

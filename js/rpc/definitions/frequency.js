export default {
  rpc: {
    frequency: {
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
  },
  types: {
    InclusionFee: {
      base_fee: "Balance",
      len_fee: "Balance",
      adjusted_weight_fee: "Balance",
    },
    FeeDetails: {
      inclusion_fee: "Option<InclusionFee>",
      tip: "Balance",
    },
  },
};

export default {
  rpc: {
    computeCapacityFeeDetails: {
      description: 'Compute the capacity fee details for a given transaction',
      params: [
        {
          name: 'encoded_xt',
          type: 'Vec<u8>',
        },
        {
          name: 'at',
          type: 'Option<BlockHash>',
        },
      ],
      type: 'FeeDetails',
    },
  },
  types: {},
};

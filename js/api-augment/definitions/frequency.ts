export default {
  rpc: {
    getEvents: {
      description: 'Get block Events',
      params: [
        {
          name: 'at',
          type: 'H256',
        },
      ],
      type: 'Vec<RpcEvent>',
    },
    getMissingNonceValues: {
      description: 'Get missing nonce values for an account',
      params: [
        {
          name: 'account',
          type: 'AccountId32',
        },
      ],
      type: 'Vec<Index>',
    },
  },
  types: {
    RpcEvent: {
      phase: 'Option<u32>',
      pallet: 'u8',
      event: 'u8',
      data: 'Vec<u8>',
    },
    // Part of the auraApi that is missing the type.
    // Unsure why, but can safely be here.
    SpConsensusSlotsSlotDuration: 'u64',
  },
};

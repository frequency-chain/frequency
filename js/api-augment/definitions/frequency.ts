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
  },
  runtime: {
    AdditionalRuntimeApi: [
      {
        methods: {
          get_events: {
            description: 'Get the events with simple SCALE decoding',
            params: [],
            type: 'Vec<RpcEvent>',
          },
        },
        version: 1,
      },
    ],
  },
};

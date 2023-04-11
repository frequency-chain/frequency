export default {
  rpc: {
    dummy: { description: 'This API has no custom RPCs', params: [], type: 'undefined' },
  },
  types: {
    RewardEra: 'u32',
    Balance: 'u128',
    BlockNumber: 'u32',
    UnclaimedRewardInfo: {
      reward_era: 'RewardEra',
      expires_at_block: 'BlockNumber',
      staked_amount: 'Balance',
      eligible_amount: 'Balance',
      earned_amount: 'Balance',
    },
  },
  runtime: {
    CapacityRuntimeApi: [
      {
        methods: {
          list_unclaimed_rewards: {
            description: 'List any rewards that can be claimed up to the previous Reward Era',
            params: [{ name: 'address', type: 'AccountId' }],
            type: 'Vec<UnclaimedRewardInfo>',
          },
        },
        version: 1,
      },
    ],
  },
};

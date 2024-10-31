export default {
  rpc: {
    dummy: { description: 'This API has no custom RPCs', params: [], type: 'undefined' },
  },
  types: {
    RewardEra: 'u32',
    UnclaimedRewardInfo: {
      reward_era: 'RewardEra',
      expires_at_block: 'BlockNumber',
      staked_amount: 'Balance',
      eligible_amount: 'Balance',
      earned_amount: 'Balance',
    },
  },
};

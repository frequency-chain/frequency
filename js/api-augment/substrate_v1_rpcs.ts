import { DefinitionRpc } from '@polkadot/types/types';

export const v1SubstrateRpcs: Record<string, Record<string, DefinitionRpc>> = {
  transactionWatch_v1: {
    submitAndWatch: {
      description: '',
      type: '',
      params: [],
    },
    unwatch: {
      description: '',
      type: '',
      params: [],
    },
  },
  transaction_v1: {
    broadcast: {
      description: '',
      type: '',
      params: [],
    },
    stop: {
      description: '',
      type: '',
      params: [],
    },
  },
  chainHead_v1: {
    body: {
      description: '',
      type: '',
      params: [],
    },
    call: {
      description: '',
      type: '',
      params: [],
    },
    continue: {
      description: '',
      type: '',
      params: [],
    },
    follow: {
      description: '',
      type: '',
      params: [],
    },
    header: {
      description: '',
      type: '',
      params: [],
    },
    stopOperation: {
      description: '',
      type: '',
      params: [],
    },
    storage: {
      description: '',
      type: '',
      params: [],
    },
    unfollow: {
      description: '',
      type: '',
      params: [],
    },
    unpin: {
      description: '',
      type: '',
      params: [],
    },
  },
};

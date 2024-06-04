export const providerUrl: string = process.env.WS_PROVIDER_URL || 'ws://localhost:9944';
export const verbose = process.env.VERBOSE_TESTS === 'true' || process.env.VERBOSE_TESTS === '1';

const CHAIN_ENVIRONMENT = {
  DEVELOPMENT: 'dev',
  ROCOCO_TESTNET: 'rococo-testnet',
  PASEO_TESTNET: 'paseo-testnet',
  ROCOCO_LOCAL: 'rococo-local',
  PASEO_LOCAL: 'paseo-local',
};

export function isTestnet() {
  switch (process.env.CHAIN_ENVIRONMENT) {
    case CHAIN_ENVIRONMENT.ROCOCO_TESTNET:
    case CHAIN_ENVIRONMENT.PASEO_TESTNET:
      return true;
  }
  return false;
}

export function isDev() {
  return process.env.CHAIN_ENVIRONMENT === CHAIN_ENVIRONMENT.DEVELOPMENT;
}

export function hasRelayChain() {
  switch (process.env.CHAIN_ENVIRONMENT) {
    case CHAIN_ENVIRONMENT.ROCOCO_TESTNET:
    case CHAIN_ENVIRONMENT.PASEO_TESTNET:
    case CHAIN_ENVIRONMENT.ROCOCO_LOCAL:
    case CHAIN_ENVIRONMENT.PASEO_LOCAL:
      return true;
  }
  return false;
}

export function getGraphChangeSchema() {
  switch (process.env.CHAIN_ENVIRONMENT) {
    case CHAIN_ENVIRONMENT.ROCOCO_TESTNET:
      return 53;
    case CHAIN_ENVIRONMENT.PASEO_TESTNET:
      return 8;
  }
  return null;
}
export function getBroadcastSchema() {
  switch (process.env.CHAIN_ENVIRONMENT) {
    case CHAIN_ENVIRONMENT.ROCOCO_TESTNET:
      return 51;
    case CHAIN_ENVIRONMENT.PASEO_TESTNET:
      return 2;
  }
  return null;
}

export function getDummySchema() {
  switch (process.env.CHAIN_ENVIRONMENT) {
    case CHAIN_ENVIRONMENT.ROCOCO_TESTNET:
      return 52;
    case CHAIN_ENVIRONMENT.PASEO_TESTNET:
      return 12;
  }
  return null;
}

export function getAvroChatMessagePaginatedSchema() {
  switch (process.env.CHAIN_ENVIRONMENT) {
    case CHAIN_ENVIRONMENT.ROCOCO_TESTNET:
      return 55;
    case CHAIN_ENVIRONMENT.PASEO_TESTNET:
      return 14;
  }
  return null;
}

export function getAvroChatMessageItemizedSchema() {
  switch (process.env.CHAIN_ENVIRONMENT) {
    case CHAIN_ENVIRONMENT.ROCOCO_TESTNET:
      return 54;
    case CHAIN_ENVIRONMENT.PASEO_TESTNET:
      return 13;
  }
  return null;
}

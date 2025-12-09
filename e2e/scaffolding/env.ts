import { ExtrinsicHelper } from './extrinsicHelpers';

export const providerUrl: string = process.env.WS_PROVIDER_URL || 'ws://localhost:9944';
export const verbose = process.env.VERBOSE_TESTS === 'true' || process.env.VERBOSE_TESTS === '1';

// Disable console output in non-verbose mode; it's far too noisy.
if (!verbose) {
  console.debug = () => {
    /* empty */
  };
  console.log = () => {
    /* empty */
  };
  console.info = () => {
    /* empty */
  };
  console.warn = () => {
    /* empty */
  };
  console.error = () => {
    /* empty */
  };
  console.count = () => {
    /* empty */
  };
}

const CHAIN_ENVIRONMENT = {
  DEVELOPMENT: 'dev',
  PASEO_TESTNET: 'paseo-testnet',
  PASEO_LOCAL: 'paseo-local',
};

export function isTestnet() {
  switch (process.env.CHAIN_ENVIRONMENT) {
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
    case CHAIN_ENVIRONMENT.PASEO_TESTNET:
    case CHAIN_ENVIRONMENT.PASEO_LOCAL:
      return true;
  }
  return false;
}

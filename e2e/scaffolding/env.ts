
export namespace env {
  export const providerUrl = process.env.WS_PROVIDER_URL;
  export const verbose = (process.env.VERBOSE_TESTS === 'true' || process.env.VERBOSE_TESTS === '1');
}


const CHAIN_ENVIRONMENT = {
  DEVELOPMENT: "dev",
  ROCOCO_TESTNET: "rococo-testnet",
  ROCOCO_LOCAL: "rococo-local",
}

export function isTestnet() {
  return process.env.CHAIN_ENVIRONMENT === CHAIN_ENVIRONMENT.ROCOCO_TESTNET;
}

export function isDev() {
  return process.env.CHAIN_ENVIRONMENT === CHAIN_ENVIRONMENT.DEVELOPMENT;
}

export function hasRelayChain() {
  return process.env.CHAIN_ENVIRONMENT === CHAIN_ENVIRONMENT.ROCOCO_LOCAL
    || process.env.CHAIN_ENVIRONMENT === CHAIN_ENVIRONMENT.ROCOCO_TESTNET
}

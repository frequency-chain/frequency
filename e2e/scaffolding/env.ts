import { ExtrinsicHelper } from './extrinsicHelpers';

export const providerUrl: string = process.env.WS_PROVIDER_URL || 'ws://localhost:9944';
export const verbose = process.env.VERBOSE_TESTS === 'true' || process.env.VERBOSE_TESTS === '1';

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

export async function getNamedIntentAndSchema(name: string) {
  const response = await ExtrinsicHelper.apiPromise.call.schemasRuntimeApi.getRegisteredEntitiesByName(name);
  if (response.isSome && response.unwrap().length > 0 && response.unwrap()[0].entityId.isIntent) {
    const intentId = response.unwrap()[0].entityId.asIntent;
    const schemaResponse = await ExtrinsicHelper.apiPromise.call.schemasRuntimeApi.getIntentById(intentId, true);
    if (schemaResponse.isSome) {
      const schemaIds = schemaResponse.unwrap().schemaIds;
      if (schemaIds.isSome && schemaIds.unwrap().length > 0) {
        const ids = schemaIds.unwrap();
        return { intentId, schemaId: ids[ids.length - 1] };
      }
    }

    return { intentId, schemaId: null };
  }

  return { intentId: null, schemaId: null };
}

export function getGraphChangeSchema() {
  return getNamedIntentAndSchema('danp.public-follows');
}
export function getBroadcastSchema() {
  return getNamedIntentAndSchema('dsnp.broadcast');
}

export function getDummySchema() {
  return getNamedIntentAndSchema('test.dummySchema');
}

export function getAvroChatMessagePaginatedSchema() {
  switch (process.env.CHAIN_ENVIRONMENT) {
    case CHAIN_ENVIRONMENT.PASEO_TESTNET:
      return 16_075;
  }
  return null;
}

export function getAvroChatMessageItemizedSchema() {
  switch (process.env.CHAIN_ENVIRONMENT) {
    case CHAIN_ENVIRONMENT.PASEO_TESTNET:
      return 16_073;
  }
  return null;
}

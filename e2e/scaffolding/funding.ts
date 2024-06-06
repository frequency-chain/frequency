import '@frequency-chain/api-augment';
import { Keyring } from '@polkadot/api';
import { isTestnet } from './env';

const coreFundingSourcesSeed = 'salt glare message absent guess transfer oblige refuse keen current lunar pilot';
const keyring = new Keyring({ type: 'sr25519' });

// This is a list of the funding sources.
// New ones should be added to support additional parallel testing
// tldr: Each test file should have a separate funding source listed below
export const fundingSources = [
  'capacity-replenishment',
  'load-signature-registry',
  'capacity-transactions',
  'sudo-transactions',
  'time-release',
  'capacity-rpcs',
  'capacity-staking',
  'capacity-unstaking',
  'schemas-create',
  'handles',
  'messages-add-ipfs',
  'misc-util-batch',
  'scenarios-grant-delegation',
  'stateful-storage-handle-sig-req',
  'msa-create-msa',
  'msa-key-management',
  'stateful-storage-handle-paginated',
  'stateful-storage-handle-itemized',
  'frequency-misc',
] as const;

// Get the correct key for this Funding Source
export function getFundingSource(name: (typeof fundingSources)[number]) {
  if (fundingSources.includes(name)) {
    return keyring.addFromUri(`${coreFundingSourcesSeed}//${name}`, { name }, 'sr25519');
  }
  throw new Error(`Unable to locate "${name}" in the list of funding sources`);
}

export function getSudo() {
  if (isTestnet()) {
    throw new Error('Sudo not available on testnet!');
  }

  return {
    uri: '//Alice',
    keys: keyring.addFromUri('//Alice'),
  };
}

export function getRootFundingSource() {
  if (isTestnet()) {
    const seed_phrase = process.env.FUNDING_ACCOUNT_SEED_PHRASE;
    if (seed_phrase === undefined) {
      console.error('FUNDING_ACCOUNT_SEED_PHRASE must not be undefined when CHAIN_ENVIRONMENT is a testnet');
      process.exit(1);
    }

    return {
      uri: 'TestnetTestRunnerAccount',
      keys: keyring.addFromMnemonic(seed_phrase),
    };
  }

  return {
    uri: '//Alice',
    keys: keyring.addFromUri('//Alice'),
  };
}

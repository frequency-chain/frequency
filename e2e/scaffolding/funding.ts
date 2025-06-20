import '@frequency-chain/api-augment';
import { Keyring } from '@polkadot/api';
import { isTestnet } from './env';
import { cryptoWaitReady } from '@polkadot/util-crypto';

const coreFundingSourcesSeed = 'salt glare message absent guess transfer oblige refuse keen current lunar pilot';
const keyring = new Keyring({ type: 'sr25519' });

// Get the correct key for this Funding Source
export function getFundingSource(name: string) {
  (async () => {
    try {
      await cryptoWaitReady();
    } catch (error) {
      console.error('Error:', error);
    }
  })();

  // Check if we are getting a full path, and if we are, chop it off
  // Every derived path should be either be a full path or relative to the e2e root
  const derivedPath = (name.includes('/e2e/') ? name.replace(/.*\/e2e\//, '') : name).replaceAll('/', '-');
  if (!derivedPath.includes('.test.ts')) {
    console.error("The requested funding source was not a test file, so it wouldn't be funded!", { derivedPath });
    throw new Error('Asked for a non-funded source');
  }
  try {
    return keyring.addFromUri(`${coreFundingSourcesSeed}//${derivedPath}`, { name: derivedPath }, 'sr25519');
  } catch (e) {
    console.error('Failed to build funding source: ', { derivedPath });
    throw e;
  }
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

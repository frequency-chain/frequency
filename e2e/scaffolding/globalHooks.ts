// These run ONCE per entire test run
import { cryptoWaitReady } from '@polkadot/util-crypto';
import { globSync } from 'glob';
import { ExtrinsicHelper } from './extrinsicHelpers';
import { getFundingSource, getRootFundingSource, getSudo } from './funding';
import {
  TEST_EPOCH_LENGTH,
  drainKeys,
  getNonce,
  setEpochLength,
  getOrCreateGraphChangeSchema,
  getOrCreateParquetBroadcastSchema,
  getOrCreateDummySchema,
  getOrCreateAvroChatMessagePaginatedSchema,
  getOrCreateAvroChatMessageItemizedSchema,
  getOrCreateIntentAndSchema,
  getOrCreateDelegationSchema,
} from './helpers';
import { isDev, providerUrl } from './env';
import { getUnifiedAddress } from '@frequency-chain/ethereum-utils';
import type { KeyringPair } from '@polkadot/keyring/types';
import { AVRO_CHAT_MESSAGE } from '../stateful-pallet-storage/fixtures/itemizedSchemaType';

const DEFAULT_AMOUNT = 100_000_000_000_000n; // 1,000,000 UNIT per source
const MINIMUM_DIFF_AMOUNT = 100_000_000n; // 1 UNIT

// This will always include files that we don't care about, but that is ok.
// The reduction in complexity is worth the extra transfers
function getAllTestFiles() {
  return globSync('**/*.test.ts', { ignore: 'node_modules/**' });
}

async function fundAccountAmount(dest: KeyringPair): Promise<{ dest: KeyringPair; amount: bigint }> {
  const accountInfo = await ExtrinsicHelper.getAccountInfo(dest);
  console.log(
    'Checking Funding: ',
    getUnifiedAddress(dest).toString(),
    'Free Balance',
    accountInfo.data.free.toHuman()
  );
  const freeBalance = accountInfo.data.free.toBigInt();

  // Only fund up to the amount needed, so that we don't have to drain the persistent accounts each time
  if (freeBalance >= DEFAULT_AMOUNT - MINIMUM_DIFF_AMOUNT) {
    return { dest, amount: 0n };
  }
  return { dest, amount: DEFAULT_AMOUNT - (freeBalance - MINIMUM_DIFF_AMOUNT) };
}

function fundSourceTransfer(root: KeyringPair, dest: KeyringPair, amount: bigint, nonce: number) {
  try {
    // Only transfer the amount needed
    return ExtrinsicHelper.transferFunds(root, dest, amount).signAndSend(nonce);
  } catch (e) {
    console.error('Unable to fund source', { dest });
    throw e;
  }
}

async function fundAccountsToDefault(dests: KeyringPair[]) {
  const root = getRootFundingSource().keys;
  console.log('Root funding source: ', getUnifiedAddress(root));
  const nonce = await getNonce(root);
  const fundingList = await Promise.all(dests.map(fundAccountAmount));
  await Promise.all(
    fundingList
      .filter(({ amount }) => amount > 0n)
      .map(({ amount, dest }, i) => fundSourceTransfer(root, dest, amount, nonce + i))
  );
  // Make sure we are finalized before trying to use the funds
  await ExtrinsicHelper.waitForFinalization();
  console.log('Root funding complete!');
}

// Pre-create named intents and schemas so we avoid double-creation errors in parallel tests
async function seedTestIntentsAndSchemas() {
  const keys = getRootFundingSource().keys;
  let nonce = await getNonce(keys);
  let { noncesUsed } = await getOrCreateGraphChangeSchema(keys, nonce);
  nonce += noncesUsed;
  ({ noncesUsed } = await getOrCreateParquetBroadcastSchema(keys, nonce));
  nonce += noncesUsed;
  ({ noncesUsed } = await getOrCreateDummySchema(keys, nonce));
  nonce += noncesUsed;
  ({ noncesUsed } = await getOrCreateAvroChatMessagePaginatedSchema(keys, nonce));
  nonce += noncesUsed;
  ({ noncesUsed } = await getOrCreateAvroChatMessageItemizedSchema(keys, nonce));
  nonce += noncesUsed;
  ({ noncesUsed } = await getOrCreateIntentAndSchema(
    keys,
    'test.ItemizedSignatureRequired',
    { payloadLocation: 'Itemized', settings: ['AppendOnly', 'SignatureRequired'] },
    { model: AVRO_CHAT_MESSAGE, modelType: 'AvroBinary' },
    nonce
  ));
  nonce += noncesUsed;
  ({ noncesUsed } = await getOrCreateIntentAndSchema(
    keys,
    'test.PaginatedSignatureRequired',
    { payloadLocation: 'Paginated', settings: ['SignatureRequired'] },
    { model: AVRO_CHAT_MESSAGE, modelType: 'AvroBinary' },
    nonce
  ));
  nonce += noncesUsed;
  ({ noncesUsed } = await getOrCreateDelegationSchema(keys, nonce, 'test.grantDelegation'));

  // Make sure we are finalized so that we don't get intent and schema creation race conditions
  await ExtrinsicHelper.waitForFinalization();
  console.log('Test intents and schemas created!');
}

async function devSudoActions() {
  // Because there is only one sudo, these actions must take place globally
  const sudo = getSudo().keys;
  await setEpochLength(sudo, TEST_EPOCH_LENGTH);
}

export async function mochaGlobalSetup(context) {
  await cryptoWaitReady();
  await ExtrinsicHelper.initialize(providerUrl);
  await fundAccountsToDefault(await Promise.all(getAllTestFiles().map(getFundingSource)));

  // Sudo is only when not on Testnet
  if (isDev()) {
    await devSudoActions();
    await seedTestIntentsAndSchemas();
  }

  console.log('Global Setup Complete');
}

export async function mochaGlobalTeardown() {
  console.log('Global Teardown Start');
  await ExtrinsicHelper.api.disconnect();
  await ExtrinsicHelper.apiPromise.disconnect();
  console.log('Global Teardown Complete');
}

import { cryptoWaitReady } from "@polkadot/util-crypto";
import { ExtrinsicHelper } from "./extrinsicHelpers";
import { fundingSources, getFundingSource, getRootFundingSource, getSudo } from "./funding";
import { TEST_EPOCH_LENGTH, getNonce, setEpochLength } from "./helpers";
import { isDev } from "./env";

const SOURCE_AMOUNT = 100_000_000_000_000n;

async function fundAllSources() {
  const root = getRootFundingSource().keys;
  const nonce = await getNonce(root);
  await Promise.all(fundingSources.map((dest, i) => {
    return ExtrinsicHelper.transferFunds(root, getFundingSource(dest), SOURCE_AMOUNT).signAndSend(nonce + i);
  }));
}

async function devSudoActions() {
  // Because there is only one sudo, these actions must take place globally
  const sudo = getSudo().keys;
  await setEpochLength(sudo, TEST_EPOCH_LENGTH);
}

async function drainAllSources() {
  const root = getRootFundingSource().keys;
  await Promise.all(fundingSources.map((source, i) => {
    return ExtrinsicHelper.emptyAccount(getFundingSource(source), root).signAndSend();
  }));
}

export async function mochaGlobalSetup() {
  console.log('Global Setup Start');
  await cryptoWaitReady();
  await ExtrinsicHelper.initialize();
  await fundAllSources();

  // Sudo is only when not on Testnet
  if (isDev()) await devSudoActions();

  console.log('Global Setup Complete');
}

export async function mochaGlobalTeardown() {
  console.log('Global Teardown Start');
  await drainAllSources();
  await ExtrinsicHelper.api.disconnect();
  await ExtrinsicHelper.apiPromise.disconnect();
  console.log('Global Teardown Complete');
}

import { cryptoWaitReady } from "@polkadot/util-crypto";
import { ExtrinsicHelper } from "./extrinsicHelpers";
import { fundingSources, getFundingSource, getRootFundingSource } from "./funding";
import { getNonce } from "./helpers";

const SOURCE_AMOUNT = 100_000_000_000_000n;

async function fundAllSources() {
  const root = getRootFundingSource().keys;
  const nonce = await getNonce(root);
  await Promise.all(fundingSources.map((dest, i) => {
    return ExtrinsicHelper.transferFunds(root, getFundingSource(dest), SOURCE_AMOUNT).signAndSend(nonce + i);
  }));
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
  console.log('Global Setup Complete');
}

export async function mochaGlobalTeardown() {
  console.log('Global Teardown Start');
  await drainAllSources();
  await ExtrinsicHelper.api.disconnect();
  await ExtrinsicHelper.apiPromise.disconnect();
  console.log('Global Teardown Complete');
}

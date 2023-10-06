import { cryptoWaitReady } from "@polkadot/util-crypto";
import { isTestnet } from "./env";
import { ExtrinsicHelper } from "./extrinsicHelpers";
import { fundingSources, getFundingSource } from "./funding";
import { createKeys, getNonce } from "./helpers";

const SOURCE_AMOUNT = 100n * 100_000_000n; // 100 * 1 UNIT

function getRootFundingSource() {
  if (isTestnet()) {
    const seed_phrase = process.env.FUNDING_ACCOUNT_SEED_PHRASE;

    if (seed_phrase === undefined) {
      console.error("FUNDING_ACCOUNT_SEED_PHRASE must not be undefined when CHAIN_ENVIRONMENT is \"rococo\"");
      process.exit(1);
    }

    return {
      uri: "RococoTestRunnerAccount",
      keys: createKeys(seed_phrase),
    };
  }

  return {
    uri: "//Alice",
    keys: createKeys("//Alice"),
  };
}

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
  console.log('Global Setup');
  await cryptoWaitReady();
  await ExtrinsicHelper.initialize();
  await fundAllSources();
}

export async function mochaGlobalTeardown() {
  console.log('Global Teardown');
  await drainAllSources();
  await ExtrinsicHelper.api.disconnect();
  await ExtrinsicHelper.apiPromise.disconnect();
}

import "@frequency-chain/api-augment";
import { Keyring } from "@polkadot/api";
import { isTestnet } from "./env";
import { createKeys } from "./helpers";

const coreFundingSourcesSeed = "salt glare message absent guess transfer oblige refuse keen current lunar pilot";
const keyring = new Keyring({ type: 'sr25519' });

// This is a list of the sources.
// The index is used to determine the
export const fundingSources = [
  "capacity-replenishment",
  "load-signature-registry",
  "capacity-transactions",
  "time-release",
  "capacity-staking",
  "schemas-create",
  "handles",
  "messages-add-ipfs",
  "misc-util-batch",
  "scenarios-grant-delegation",
  "stateful-storage-handle-sig-req",
  "msa-create-msa",
  "stateful-storage-handle-paginated",
  "stateful-storage-handle-itemized",
] as const;

// Get the correct key for this Funding Source
export function getFundingSource(name: typeof fundingSources[number]) {
  if (fundingSources.includes(name)) {
    return keyring.addFromUri(`${coreFundingSourcesSeed}//${name}`, { name }, 'sr25519');
  }
  throw new Error(`Unable to locate "${name}" in the list of funding sources`);
}

export function getSudo() {
  if (isTestnet()) {
    throw new Error("Sudo not available on testnet!")
  }

  return {
    uri: "//Alice",
    keys: createKeys("//Alice"),
  };
}

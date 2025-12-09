import "@frequency-chain/api-augment";
import { ApiPromise, Keyring, WsProvider } from "@polkadot/api";
import { blake2AsU8a, xxhashAsU8a } from "@polkadot/util-crypto";
import { u8aConcat, u8aToHex, stringToU8a } from "@polkadot/util";
import fs from "fs/promises";
import { HexString } from "@polkadot/util/types";
import { parseArgs } from "node:util";
import { childKey } from "@polkadot/api-derive/crowdloan";

// Monkey-patch BigInt so that JSON.stringify will work
// eslint-disable-next-line
BigInt.prototype["toJSON"] = function () {
  return this.toString();
};

// ---------- constants: adjust as needed ----------
const PALLET_NAME = "stateful-storage";
const STORAGE_NAMES = ["paginated", "itemized"] as const;

const CHILD_TRIE_DATA_OUTFILE = "child-trie-data.json";
const CHILD_ROOT_OUTFILE = "child-roots.json";
const CHILD_KEYS_OUTFILE = "child-keys.json";

// ---------- helpers ----------
const CHILD_PREFIX = stringToU8a(":child_storage:default:"); // matches DEFAULT_CHILD_STORAGE_KEY_PREFIX from substrate

interface ChildTrieKey {
  intentId?: number;
  pageId?: number;
  key: HexString;
}

interface ChildTrie {
  msaId: bigint;
  storageName: "paginated" | "itemized";
  unprefixedChildRootHex: HexString;
  prefixedChildRootHex: HexString;
  prefixedChildRoot: Uint8Array;
  childKeys: ChildTrieKey[] | undefined;
}

let api: ApiPromise;

function deriveChildTrieRoot(api: ApiPromise, id: bigint, segment: string, storageName: string) {
  const msaId = api.registry.createType("u64", id).toU8a(true);
  const msaKey = new Uint8Array(Buffer.concat([msaId, stringToU8a("::")]));
  const a = blake2AsU8a(msaKey, 128);
  const b = xxhashAsU8a(stringToU8a(segment), 64); // 8 bytes
  const c = xxhashAsU8a(stringToU8a(storageName), 64); // 8 bytes
  const id32 = u8aConcat(a, b, c); // 32 bytes
  return { a, b, c, id32 };
}

function childStorageKey(childId32: Uint8Array): Uint8Array {
  return u8aConcat(CHILD_PREFIX, childId32);
}

function toHex(u: Uint8Array): HexString {
  return u8aToHex(u) as HexString;
}

// Function to extract the key values out of the child key
// `stateful-storage` child keys are hashed using a "concat" hash,
// which places the original (SCALE-encoded) key tuple after the hashed key parts.
// The 'arity' is the number of key-parts in the tuple. Each key-part is hashed
// with a twox_128 hash. So the resulting key looks like:
//
// (twox_128_1, ..., twox_128_n, u16_1, ..., u16_n)
function decodeChildKey(api: ApiPromise, key: HexString): ChildTrieKey {
  const childKey: ChildTrieKey = { key };
  const keyBytes = new Uint8Array(Buffer.from(key.replace(/^0x/, ""), "hex"));

  // Key is either:
  // (twox_128 + twox_128 + u16 + 16)
  // (twox_128 + u16)
  switch (keyBytes.length) {
    case 36:
      const decodedKey1 = api.registry.createType('(u128, u128, u16, u16)', keyBytes);
      childKey.intentId = decodedKey1.at(2)?.toNumber();
      childKey.pageId = decodedKey1.at(3)?.toNumber();
      break;

    case 18:
      const decodedKey2 = api.registry.createType('(u128, u16)', keyBytes);
      childKey.intentId = decodedKey2.at(1)?.toNumber();
      childKey.pageId = 0;
      break;

    default:
      console.error(`Invalid key length: ${keyBytes.length}`);
      break;
  }

  return childKey;
}
async function childGetKeys(api: ApiPromise, childKeyHex: string, prefixHex: string): Promise<ChildTrieKey[]> {
  const keys: ChildTrieKey[] = [];

  // NOTE: Using `getKeys` here instead of `getKeysPaged` because, at time of writing, there's not that
  // much data per-user on-chain to warrant paging. If that changes, may need to switch to a paged fetch.
  const batch: any = await api.rpc.childstate.getKeys(childKeyHex, prefixHex);
  const arr = batch.map((k: any) => decodeChildKey(api, k.toString()));
  keys.push(...arr);
  return keys;
}

async function resolveEndInclusive(api: ApiPromise, end: string | undefined): Promise<bigint> {
  const maxMsaId: bigint = (await api.query.msa.currentMsaIdentifierMaximum()).toBigInt();
  if (!end || BigInt(end) > maxMsaId) {
    return maxMsaId;
  }

  return BigInt(end);
}

// Helper function to seed some data for testing on a local chain
async function createTestData(api: ApiPromise) {
  const keys = new Keyring({ type: "sr25519" }).createFromUri("//Alice");
  await new Promise((resolve, reject) =>
    api.tx.msa.create().signAndSend(keys, ({ events = [], status }) => {
      if (status.isFinalized) {
        console.log("Transaction finalized at blockHash", status.asFinalized.toHex());
        events.forEach(({ event: { data, method, section } }) => {
          console.log("\t", section, ".", method, data.toString());
        });
        resolve(true);
      }
    }),
  );
  const msaResult = await api.query.msa.publicKeyToMsaId(keys.address);
  const msaId = msaResult.unwrap().toBigInt();

  const payload = new Uint8Array(1024).fill(0x01);

  await new Promise((resolve, reject) =>
    api.tx.statefulStorage.upsertPage(msaId, 8, 2, 0, payload).signAndSend(keys, ({ events = [], status }) => {
      if (status.isFinalized) {
        console.log("Transaction finalized at blockHash", status.asFinalized.toHex());
        events.forEach(({ event: { data, method, section } }) => {
          console.log("\t", section, ".", method, data.toString());
        });
        resolve(true);
      }
    }),
  );
}

// ---------- main ----------
async function main() {

  const { values } = parseArgs({
    args: process.argv.slice(2),
    options: {
      'start-msa-id': {
        type: 'string',
        short: 's',
        default: '1',
      },
      'end-msa-id': {
        type: 'string',
        short: 'e'
      },
      'collect-keys': {
        type: 'boolean',
        short: 'k'
      },
      'page-size': {
        type: 'string',
        short: 'p',
        default: '10'
      },
      'uri': {
        type: 'string',
        short: 'u',
        default: 'ws://127.0.0.1:9944',
      },
      'create-test-data': {
        type: 'boolean',
        short: 't',
        default: false,
      }
    },
    allowNegative: true,
  })
  const provider = new WsProvider(values.uri);
  api = await ApiPromise.create({ provider });

  if (values['create-test-data']) {
    await createTestData(api);
  }

  const start = BigInt(values['start-msa-id']);
  const endInclusive = await resolveEndInclusive(api, values['end-msa-id']);
  console.log(`Resolved end-inclusive to ${endInclusive}`);

  console.log(`Fetching child containers ${start} to ${endInclusive}...`);

  let count = 0;
  let childKeys: string[] = [];
  let rootsWithKeys: string[] = [];

  // Create child roots to fetch
  const computed_roots: ChildTrie[] = [];
  for (let id = start; id <= endInclusive; ++id) {
    STORAGE_NAMES.forEach((storageName) => {
      const { id32 } = deriveChildTrieRoot(api, id, PALLET_NAME, storageName);
      const unprefixedChildRootHex = toHex(id32);
      const prefixedChildRoot = childStorageKey(id32);
      const prefixedChildRootHex = toHex(prefixedChildRoot);
      computed_roots.push({
        msaId: id,
        storageName,
        unprefixedChildRootHex,
        prefixedChildRoot,
        prefixedChildRootHex,
        childKeys: undefined,
      });
    });
  }

  let childRoots: string[] = [];
  let fetchedData: ChildTrie[] = [];
  if (!values['collect-keys']) {
    childRoots = computed_roots.map((r) => r.unprefixedChildRootHex);
  }
  else {
    while (computed_roots.length > 0) {
      const roots_to_process = computed_roots.splice(0, Number(values['page-size']));
      if (count > 0 && count % 1000 === 0) {
        console.log("Fetched ", count, " child tries.");
      }

      const promises =
        roots_to_process.flatMap(async (root) => {
          const childKeys = await childGetKeys(api, root.prefixedChildRootHex, '0x');
          return { ...root, childKeys };
        });
      const fetchedKeys = (await Promise.all(promises)).filter((e) => e.childKeys && e.childKeys.length > 0);
      rootsWithKeys = rootsWithKeys.concat(fetchedKeys.map((k) => k?.unprefixedChildRootHex))
      childKeys = childKeys.concat(fetchedKeys.flatMap((k) => k?.childKeys).map((k) => k.key));
      fetchedData = fetchedData.concat(fetchedKeys);
      count += roots_to_process.length;
    }

    childRoots = rootsWithKeys;
  }

  console.log('Preparing output...');

  // Prepare consolidated output showing intents that contain data & list of MSAs that have data in that intent
  type MsaPagesMap = Map<bigint, number[]>;
  const consolidatedSnapshot = fetchedData.reduce((acc, cur) => {
    const key = `${cur.storageName}:`;
    cur.childKeys?.forEach((childKey) => {
      const keyStr = `${key}${childKey.intentId}`
      const value = acc.get(keyStr) || new Map<bigint, number[]>();
      const childValue = value.get(cur.msaId) || [];
      childValue.push(childKey.pageId!);
      value.set(cur.msaId, childValue);
      acc.set(keyStr, value);
    });
    return acc;
  }, new Map<string, MsaPagesMap>());

  // Convert map to a structured array suitable for serialization
  const serializedData: any = {};
  consolidatedSnapshot.forEach((value, key) => {
    const arr: bigint[] = [];
    value.forEach((pages, msaId) => {
      arr.push(msaId);
    })
    serializedData[key] = arr;
  });

  await fs.writeFile(CHILD_TRIE_DATA_OUTFILE, JSON.stringify(serializedData, null, 2), { encoding: "utf-8" });
  console.log(`Wrote ${CHILD_TRIE_DATA_OUTFILE} with ${Object.keys(serializedData).length} data-containing intents.`);

  // Write list of child roots
  // (if key fetch option enabled, only roots containing keys will be listed; otherwise the entire set of computed roots will be listed)
  await fs.writeFile(
    CHILD_ROOT_OUTFILE,
    JSON.stringify(
      childRoots,
    ),
    { encoding: "utf-8" },
  );
  console.log(`Wrote ${CHILD_ROOT_OUTFILE} with ${childRoots.length} roots.`);

  // Write list of discovered child keys (empty if key fetch option not enabled)
  await fs.writeFile(
    CHILD_KEYS_OUTFILE,
    JSON.stringify(childKeys), { encoding: "utf-8" }
  );
  console.log(`Wrote ${CHILD_KEYS_OUTFILE} with ${childKeys.length} child keys.`);
}

const keysStr = await fs.readFile(CHILD_KEYS_OUTFILE, { encoding: "utf-8" });
const keys = JSON.parse(keysStr);

let newKeys = keys.map((k) => {
  const key = new Uint8Array(Buffer.from(k.replace(/^0x/, ""), "hex"));
  return toHex(childStorageKey(key));
});
newKeys = Array.from(new Set(newKeys));
console.log(`Found ${newKeys.length} unique child keys from original ${keys.length}`);
await fs.writeFile(`${CHILD_KEYS_OUTFILE}.2`, JSON.stringify(newKeys), { encoding: "utf-8" });
process.exit(0);
main()
  .catch((e) => {
    console.error(e);
    process.exit(1);
  })
  .finally(async () => await api.disconnect());

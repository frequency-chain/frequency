// Only way to silence PolkadotJS API warnings we don't want
console.warn = () => {};

import { ApiPromise, WsProvider } from "@polkadot/api";

const SOURCE_URL = "wss://1.rpc.frequency.xyz";
const BATCH_SIZE = 1000;

const options = {
	rpc: {
		msa: {
			getKeysByMsaId: {
				description: 'Fetch Keys for an MSA Id',
				params: [
					{
						name: 'msa_id',
						type: 'MessageSourceId',
					},
				],
				type: 'Option<KeyInfoResponse>',
			},
		},
	},
	types: {
		MessageSourceId: 'u64',
		KeyInfoResponse: {
			msa_keys: 'Vec<AccountId>',
			msa_id: 'MessageSourceId',
		},
	},
};

export async function fetchControlKeysFromState(sourceUrl) {
	// Connect to the state source
	const sourceProvider = new WsProvider(sourceUrl);
	const sourceApi = await ApiPromise.create({provider: sourceProvider, ...options});

	// msa pallet + publicKeyToMsaId
	const prefixKeys = "0x9f76716a68a582c703dd9e44700429b927e2ea47730fb1c9d7cedc4c78d1d67e";

	let result = await sourceApi.rpc.state.getKeysPaged(prefixKeys, BATCH_SIZE);

	while (result.length === BATCH_SIZE) {
		await printPublicKeysWithMsaId(sourceApi, getPublicKeys(result));

		const lastKey = result[result.length - 1];
		result = await sourceApi.rpc.state.getKeysPaged(prefixKeys, BATCH_SIZE, lastKey);
	}

	await printPublicKeysWithMsaId(sourceApi, getPublicKeys(result));
}

function getPublicKeys(results) {
	const publicKeys = [];
	for (const r of results) {
		const wholeKey = r.toHex();
		const publicKey = wholeKey.substring(wholeKey.length - 64);
		publicKeys.push(`0x${publicKey}`);
	}
	return publicKeys;
}

export async function printPublicKeysWithMsaId(sourceApi, publicKeys) {
	let promises = [];
	for (const publicKey of publicKeys) {
		promises.push(sourceApi.query.msa.publicKeyToMsaId(publicKey));
	}

	if (promises.length > 0) {
		printResults(await Promise.all(promises), publicKeys);
	}
}

function printResults(results, keys) {
	for (let i = 0; i < results.length ; i++) {
		const r = results[i];
		const k = keys[i];
		if (!r.isSome) {
			console.error(`No MsaId for ${k}`);
		} else {
			const msaId = r.unwrap();
			console.log(`${msaId},${k}`);
		}
	}
}


async function main() {
	try {
		await fetchControlKeysFromState(SOURCE_URL);
		process.exit(0);
	} catch (error) {
		console.error("Error:", error);
		process.exit(1);
	}
}

main();

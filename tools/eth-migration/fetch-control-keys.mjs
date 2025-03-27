// Only way to silence PolkadotJS API warnings we don't want
console.warn = () => {};

import { ApiPromise, WsProvider } from "@polkadot/api";

const SOURCE_URL = process.env["FREQUENCY_URL"] || "wss://1.rpc.frequency.xyz";
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

	let startKey;
	let result;
	do {
		result = await sourceApi.query.msa.publicKeyToMsaId.entriesPaged({
			args: [],
			pageSize: BATCH_SIZE,
			startKey
		});

		result.forEach(([key, value]) => {
			if (!value.isSome) {
				console.error(`No MsaId for ${key.args[0].toHex()}`);
			} else {
				console.log(`${value.unwrap().toString()},${key.args[0].toHex()}`);
			}
		});

		if (result.length > 0) {
			startKey = result[result.length - 1][0];
		}
	} while (result.length > 0);
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

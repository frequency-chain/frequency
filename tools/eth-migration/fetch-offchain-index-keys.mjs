// Only way to silence PolkadotJS API warnings we don't want
console.warn = () => {};

import { ApiPromise, WsProvider } from "@polkadot/api";

const SOURCE_URL = "wss://1.rpc.frequency.xyz";
const PROMISE_BATCH_SIZE = 500;

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

export async function fetchControlKeysFromOffchainRpc(sourceUrl) {
	// Connect to the state source
	const sourceProvider = new WsProvider(sourceUrl);
	const sourceApi = await ApiPromise.create({provider: sourceProvider, ...options});

	// Get the max schema id
	let msaMaxId = (await sourceApi.query.msa.currentMsaIdentifierMaximum()).toNumber();
	let promises = [];
	let id = 1;
	let batchNumber = 0;
	for (; id <= msaMaxId ; id++){
		promises.push(sourceApi.rpc.msa.getKeysByMsaId(id));
		if (id % PROMISE_BATCH_SIZE === 0 && promises.length > 0) {
			printRpcResults(await Promise.all(promises), batchNumber);
			promises = [];
			msaMaxId = (await sourceApi.query.msa.currentMsaIdentifierMaximum()).toNumber();
			batchNumber++;
		}
	}

	if (promises.length > 0) {
		printRpcResults(await Promise.all(promises), batchNumber);
	}
}

function printRpcResults(results, batchNumber) {
	let index = 0;
	for (const r of results) {
		if (!r.isSome) {
			console.error(`No keys for MsaId: ${(batchNumber * PROMISE_BATCH_SIZE + index)}!`);
		} else {
			const keys = r.unwrap();
			for (const key of keys.msa_keys){
				console.log(`${keys.msa_id},${key.toHex()}`);
			}
		}
		index ++;
	}
}



async function main() {
	try {
		await fetchControlKeysFromOffchainRpc(SOURCE_URL);
		process.exit(0);
	} catch (error) {
		console.error("Error:", error);
		process.exit(1);
	}
}

main();

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

// 0x26aa394eea5630e07c48ae0c9558cef7
//   b99d880ec681799c0cf30e8886371da9
//   005c2cfa803bd987e8f26e9843d3f16f
//   a29da12763d36885f88f7553561d06ad462c478c1577892e1fe273233717b172

export async function fetchTokenKeysFromState(sourceUrl) {
	// Connect to the state source
	const sourceProvider = new WsProvider(sourceUrl);
	const sourceApi = await ApiPromise.create({provider: sourceProvider, ...options});

	// system pallet + account
	const prefixKeys = "0x26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da9";

	let result = await sourceApi.rpc.state.getKeysPaged(prefixKeys, BATCH_SIZE);

	while (result.length === BATCH_SIZE) {
		await printPublicKeysAndTokens(sourceApi, getPublicKeys(result));

		const lastKey = result[result.length - 1];
		result = await sourceApi.rpc.state.getKeysPaged(prefixKeys, BATCH_SIZE, lastKey);
	}

	await printPublicKeysAndTokens(sourceApi, getPublicKeys(result));
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

export async function printPublicKeysAndTokens(sourceApi, publicKeys) {
	let accountPromises = [];
	let controlKeyPromises = [];
	for (const publicKey of publicKeys) {
		accountPromises.push(sourceApi.query.system.account(publicKey));
		controlKeyPromises.push(sourceApi.query.msa.publicKeyToMsaId(publicKey));
	}

	if (accountPromises.length > 0) {
		printResults(publicKeys, await Promise.all(accountPromises), await Promise.all(controlKeyPromises));
	}
}

// {
// 	"publicKey": "0x488e14e049fd59b77e5821c4bb71fc9c4b8ced4c19ed27b3949e91b6274f1400",
// 	"values": {
// 		"nonce": 0,
// 			"consumers": 0,
// 			"providers": 1,
// 			"sufficients": 0,
// 			"data": {
// 			"free": 5000000000,
// 			"reserved": 0,
// 			"frozen": 0,
// 			"flags": "0x80000000000000000000000000000000"
// 		}
// 	}
// }
function printResults(publicKeys, accountResults, controlKeyResult) {
	for (let i = 0; i < accountResults.length ; i++) {
		const obj = {
			publicKey: publicKeys[i],
			msaId: controlKeyResult[i].isSome ? controlKeyResult[i].unwrap().toNumber() : 0,
			values: accountResults[i],
		};
		console.log(`${JSON.stringify(obj)}`);
	}
}


async function main() {
	try {
		await fetchTokenKeysFromState(SOURCE_URL);
		process.exit(0);
	} catch (error) {
		console.error("Error:", error);
		process.exit(1);
	}
}

main();

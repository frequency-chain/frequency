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

// {
// 	"publicKey": "0x488e14e049fd59b77e5821c4bb71fc9c4b8ced4c19ed27b3949e91b6274f1400",
//  "msaId": 1,
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
export async function fetchTokenKeysFromState(sourceUrl) {
	// Connect to the state source
	const sourceProvider = new WsProvider(sourceUrl);
	const sourceApi = await ApiPromise.create({provider: sourceProvider, ...options});

	let result;
	do {
		result = await sourceApi.query.system.account.entriesPaged({ args: [], pageSize: BATCH_SIZE, startKey: result && result[result.length - 1][0] });
		let msaResult = await sourceApi.query.msa.publicKeyToMsaId.multi(result.map(([key, _]) => key.args[0]));
		result.forEach(([key, account], index) => {
			const obj = {
				publicKey: key.args[0].toHex(),
				msaId: msaResult[index].isSome ? msaResult[index].unwrap().toNumber() : 0,
				values: account,
			}
			console.log(`${JSON.stringify(obj)}`);
		});
	} while (result.length === BATCH_SIZE);
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

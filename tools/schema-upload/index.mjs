import { ApiPromise, WsProvider, Keyring } from "@polkadot/api";
import fs from "fs";

const MAINNET_SOURCE_URL = "wss://1.rpc.frequency.xyz";
const PASEO_SOURCE_URL = "wss://0.rpc.testnet.amplica.io";
const LOCAL_SOURCE_URL = "ws://127.0.0.1:9944";
const MAINNET = "MAINNET";
const PASEO = "PASEO";
const LOCAL = "LOCAL";
const DOMAN = "ics";
const INTENTS = [
	{
		payload_location: "Itemized",
		settings: ["AppendOnly", "SignatureRequired"],
		name: "ics.public-key-key-agreement",
	},
	{
		payload_location: "Itemized",
		settings: ["SignatureRequired"],
		name: "ics.context-group-acl",
	},
	{
		payload_location: "Paginated",
		settings: ["SignatureRequired"],
		name: "ics.context-group-metadata",
	},
];
const SCHEMAS = [
	{
		intent_name: "ics.public-key-key-agreement",
		model_type: "AvroBinary",
		payload_location: "Itemized",
		status: "Active",
		model: '{"type":"record","name":"PublicKey","namespace":"ics","fields":[{"name":"publicKey","doc":"Multicodec public key","type":"bytes"}]}',
	},
	{
		intent_name: "ics.context-group-acl",
		model_type: "AvroBinary",
		payload_location: "Itemized",
		status: "Active",
		model: '{"type":"record","name":"ContextGroupACL","namespace":"ics","fields":[{"name":"prid","type":"fixed","size":8,"doc":"Pseudonymous Relationship Identifier"},{"name":"keyId","type":"long","doc":"User-Assigned Key Identifier used for PRID and encryption"},{"name":"nonce","type":"fixed","size":12,"doc":"Nonce used in encryptedProviderMsaId encryption (12 bytes)"},{"name":"encryptedProviderId","type":"bytes","maxLength":10,"doc":"Encrypted provider Msa id"}]}',
	},
	{
		intent_name: "ics.context-group-metadata",
		model_type: "AvroBinary",
		payload_location: "Paginated",
		status: "Active",
		model: '{"type":"record","name":"ContextGroupMetadata","namespace":"ics","fields":[{"name":"prid","type":"fixed","size":8,"doc":"Pseudonymous Relationship Identifier"},{"name":"keyId","type":"long","doc":"User-Assigned Key Identifier used for PRID"},{"name":"locationUri","type":"string","maxLength":800,"doc":"URI pointing to the location of stored Context Group"},{"name":"contentHash","type":["null","string"],"default":null,"maxLength":128,"doc":"Optional multihash of the content in base58 encoding"}]}',
	},
];

const RPC_AUGMENTS = {
	rpc: {
		schemas: {
			getBySchemaId: {
				description: "Get a Schema by Id",
				params: [
					{
						name: "schema_id",
						type: "SchemaId",
					},
				],
				type: "Option<SchemaResponse>",
			},
		},
	},
	types: {
		SchemaId: "u16",
		SchemaModel: "Vec<u8>",
		SchemaVersion: "u8",
		SchemaResponse: {
			schema_id: "SchemaId",
			model: "SchemaModel",
			model_type: "ModelType",
			payload_location: "PayloadLocation",
			settings: "Vec<SchemaSetting>",
		},
		IntentGroupId: "u16",
		IntentGroupResponse: {
			intent_group_id: "IntentGroupId",
			intent_ids: "Vec<IntentId>",
		},
		IntentId: "u16",
		IntentResponse: {
			intent_id: "IntentId",
			payload_location: "PayloadLocation",
			settings: "Vec<IntentSetting>",
			schema_ids: "Option<Vec<SchemaId>>",
		},
		IntentSetting: {
			_enum: ["AppendOnly", "SignatureRequired"],
		},
		MappedEntityIdentifier: {
			_enum: {
				Intent: "IntentId",
				IntentGroup: "IntentGroupId",
			},
		},
		ModelType: {
			_enum: ["AvroBinary", "Parquet"],
		},
		NameLookupResponse: {
			name: "String",
			entity_id: "MappedEntityIdentifier",
		},
		PayloadLocation: {
			_enum: ["OnChain", "IPFS", "Itemized", "Paginated"],
		},
		SchemaInfoResponse: {
			schema_id: "SchemaId",
			intent_id: "IntentId",
			model_type: "ModelType",
			status: "SchemaStatus",
			payload_location: "PayloadLocation",
			settings: "Vec<IntentSetting>",
		},
		SchemaResponseV2: {
			schema_id: "SchemaId",
			intent_id: "IntentId",
			model: "SchemaModel",
			model_type: "ModelType",
			payload_location: "PayloadLocation",
			settings: "Vec<IntentSetting>",
			status: "SchemaStatus",
		},
		SchemaSetting: {
			_enum: ["AppendOnly", "SignatureRequired"],
		},
		SchemaStatus: {
			_enum: ["Active", "Deprecated", "Unsupported"],
		},
		SchemaVersionResponse: {
			schema_name: "String",
			schema_version: "SchemaVersion",
			schema_id: "SchemaId",
		},
	},
};

// DEPLOY_SCHEMA_ACCOUNT_URI (environment variable)
// The value is a URI for the account.  e.g. //Alice or a mnemonic (seed words)
const getSignerAccountKeys = () => {
	const keyring = new Keyring();

	let DEPLOY_SCHEMA_ACCOUNT_URI = process.env.DEPLOY_SCHEMA_ACCOUNT_URI;
	if (DEPLOY_SCHEMA_ACCOUNT_URI === undefined) {
		DEPLOY_SCHEMA_ACCOUNT_URI = "//Alice";
	}
	return keyring.addFromUri(DEPLOY_SCHEMA_ACCOUNT_URI, {}, "sr25519");
};

// async function getSchemas(domain, api) {
//   const nameToIds = await api.rpc.schemas.getVersions(domain);

//   const names = [];

//   for (const v of JSON.parse(JSON.stringify(nameToIds))) {
//     names.push(v.schema_name);
//   }

//   return names;
// }

function readSchema(filename) {
	const content = fs.readFileSync(filename, "utf8");
	const jsonData = JSON.parse(content);
	return jsonData;
}

async function deploy(chainType) {
	const selectedUrl =
		chainType === MAINNET
			? MAINNET_SOURCE_URL
			: chainType === PASEO
				? PASEO_SOURCE_URL
				: LOCAL_SOURCE_URL;
	console.log(`Selected CHAIN: ${chainType}, URL: ${selectedUrl}`);

	// get existing schemas on chain
	const provider = new WsProvider(selectedUrl);
	const api = await ApiPromise.create({
		provider,
		throwOnConnect: true,
		...RPC_AUGMENTS,
	});
	// const existingNames = await getSchemas(DOMAN, api);
	// console.log(existingNames);

	// upload schemas that don't exist on chain
	const signerAccountKeys = getSignerAccountKeys();
	// Retrieve the current account nonce so we can increment it when submitting transactions
	const baseNonce = (await api.rpc.system.accountNextIndex(signerAccountKeys.address)).toNumber();

	const promises = [];
	for (const idx in INTENTS) {
		const intent = INTENTS[idx];
		const nonce = baseNonce + Number(idx);
		if (chainType === MAINNET) {
			// create proposal
		} else {
			// create directly via sudo
			promises[idx] = getIntentSudoTransaction(api, signerAccountKeys, nonce, intent);
		}
	}
	// for (const idx in SCHEMAS) {
	//   const schema = SCHEMAS[idx];
	//   const schemaFullName = `${DOMAN}.${schema.nameWithoutDomain}`;
	//   if (existingNames.indexOf(schemaFullName) !== -1) {
	//     console.log(
	//       `schema ${schemaFullName} already exists! Skip adding to the chain!`,
	//     );
	//     continue;
	//   }

	//   const nonce = baseNonce + Number(idx);
	//   const json = JSON.stringify(schema.model);
	//   // Remove whitespace in the JSON
	//   const json_no_ws = JSON.stringify(JSON.parse(json));

	//   if (chainType === MAINNET) {
	//     // create proposal
	//     promises[idx] = getProposalTransaction(
	//       api,
	//       signerAccountKeys,
	//       nonce,
	//       schema,
	//       json_no_ws,
	//     );
	//   } else {
	//     // create directly via sudo
	//     promises[idx] = getSudoTransaction(
	//       api,
	//       signerAccountKeys,
	//       nonce,
	//       schema,
	//       json_no_ws,
	//     );
	//   }
	// }
	await Promise.all(promises);
}

// Given a list of events, a section and a method,
// returns the first event with matching section and method.
const eventWithSectionAndMethod = (events, section, method) => {
	const evt = events.find(({ event }) => event.section === section && event.method === method);
	return evt?.event;
};

function getSchemaProposalTransaction(api, signerAccountKeys, nonce, schemaDeploy, json_no_ws) {
	// Propose to create
	const promise = new Promise((resolve, reject) => {
		api.tx.schemas
			.proposeToCreateSchemaV2(
				json_no_ws,
				schemaDeploy.model_type,
				schemaDeploy.payload_location,
				schemaDeploy.settings,
				`${DOMAN}.${schemaDeploy.nameWithoutDomain}`,
			)
			.signAndSend(signerAccountKeys, { nonce }, ({ status, events, dispatchError }) => {
				if (dispatchError) {
					console.error("ERROR: ", dispatchError.toHuman());
					console.log("Might already have a proposal with the same hash?");
					reject(dispatchError.toHuman());
				} else if (status.isInBlock || status.isFinalized) {
					const evt = eventWithSectionAndMethod(events, "council", "Proposed");
					if (evt) {
						const id = evt?.data[1];
						const hash = evt?.data[2].toHex();
						console.log(
							"SUCCESS: " +
								`${DOMAN}.${schemaDeploy.nameWithoutDomain}` +
								" schema proposed with id of " +
								id +
								" and hash of " +
								hash,
						);
						resolve((id, hash));
					} else {
						const err = "Proposed event not found";
						console.error(`ERROR: ${err}`);
						reject(err);
					}
				}
			});
	});
	return promise;
}

function getSchemaSudoTransaction(api, signerAccountKeys, nonce, schemaDeploy, json_no_ws) {
	// Create directly via sudo
	const tx = api.tx.schemas.createSchemaViaGovernanceV2(
		signerAccountKeys.address,
		json_no_ws,
		schemaDeploy.model_type,
		schemaDeploy.payload_location,
		schemaDeploy.settings,
		`${DOMAN}.${schemaDeploy.nameWithoutDomain}`,
	);
	const promise = new Promise((resolve, reject) => {
		api.tx.sudo
			.sudo(tx)
			.signAndSend(signerAccountKeys, { nonce }, ({ status, events, dispatchError }) => {
				if (dispatchError) {
					console.error("ERROR: ", dispatchError.toHuman());
					reject(dispatchError.toHuman());
				} else if (status.isInBlock || status.isFinalized) {
					const evt = eventWithSectionAndMethod(events, "schemas", "SchemaCreated");
					if (evt) {
						const id = evt?.data[1];
						console.log(
							"SUCCESS: " +
								`${DOMAN}.${schemaDeploy.nameWithoutDomain}` +
								" schema created with id of " +
								id,
						);
						resolve((id, null));
					} else {
						const err = "SchemaCreated event not found";
						console.error(`ERROR: ${err}`);
						reject(err);
					}
				}
			});
	});
	return promise;
}

function getIntentSudoTransaction(api, signerAccountKeys, nonce, intentDeploy) {
	// Create directly via sudo
	const tx = api.tx.schemas.createIntentViaGovernance(
		signerAccountKeys.address,
		intentDeploy.payload_location,
		intentDeploy.settings,
		intentDeploy.name,
	);
	const promise = new Promise((resolve, reject) => {
		api.tx.sudo
			.sudo(tx)
			.signAndSend(signerAccountKeys, { nonce }, ({ status, events, dispatchError }) => {
				if (dispatchError) {
					console.error("ERROR: ", dispatchError.toHuman());
					reject(dispatchError.toHuman());
				} else if (status.isInBlock || status.isFinalized) {
					const evt = eventWithSectionAndMethod(events, "schemas", "IntentCreated");
					if (evt) {
						const id = evt?.data[1];
						console.log(
							"SUCCESS: " + intentDeploy.name + " intent created with id of " + id,
						);
						resolve((id, null));
					} else {
						const err = "SchemaCreated event not found";
						console.error(`ERROR: ${err}`);
						reject(err);
					}
				}
			});
	});
	return promise;
}

async function main() {
	try {
		console.log("Uploading Intents");
		const args = process.argv.slice(2);
		if (args.length == 0) {
			console.log(`Chain type should be provided: ${MAINNET} or ${PASEO} or ${LOCAL}`);
			process.exit(1);
		}
		const chainType = args[0].toUpperCase().trim();
		if (chainType !== MAINNET && chainType !== PASEO && chainType !== LOCAL) {
			console.log(`Please specify the chain type: ${MAINNET} or ${PASEO} or ${LOCAL}`);
			process.exit(1);
		}

		await deploy(chainType);
		process.exit(0);
	} catch (error) {
		console.error("Error:", error);
		process.exit(1);
	}
}

main();

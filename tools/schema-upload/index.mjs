import { ApiPromise, WsProvider, Keyring } from "@polkadot/api";

const MAINNET_SOURCE_URL = "wss://1.rpc.frequency.xyz";
const PASEO_SOURCE_URL = "wss://0.rpc.testnet.amplica.io";
const LOCAL_SOURCE_URL = "ws://127.0.0.1:9944";
const MAINNET = "MAINNET";
const PASEO = "PASEO";
const LOCAL = "LOCAL";
const INTENT = "INTENT";
const SCHEMA = "SCHEMA";
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
	{
		payload_location: "OnChain",
		settings: [],
		name: "ics.context-batch-announcement"
	}
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
	{
		intent_name: "ics.context-batch-announcement",
		model_type: "AvroBinary",
		payload_location: "OnChain",
		status: "Active",
		model: '{"type":"record","name":"ContextBatchAnnouncement","namespace":"ics","fields":[{"name":"batchHash","type":"fixed","size":32,"doc":"SHA-256 hash of the batch file for verification"},{"name":"opsCount","type":"int","doc":"Number of top-level records in the batch file"},{"name":"byteCount","type":"int","doc":"File size of the batch file in bytes"}]}',
	}
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

function getIntentId(api, intent) {
	// Propose to create
	const promise = new Promise((resolve, reject) => {
		api.call.schemasRuntimeApi.getRegisteredEntitiesByName(intent.name)
			.then(response => { 
				if (response.isSome && response.unwrap().length > 0) {
					const registeredEntities = response.unwrap().toArray();
					const last = registeredEntities[registeredEntities.length - 1];
					const id = last.entityId;
					resolve(id.value);
				} else { 
					console.log(`No intent exists for "${intent.name}"`);
					resolve(undefined);
				}
			})
			.catch(error => {
				console.error(`ERROR: ${error}`);
		        reject(error);
		    });
	});
	return promise;
}

async function hasDuplicateLatestSchema(api, schemaDeploy, intentId) {
	const intentResponse = await api.call.schemasRuntimeApi.getIntentById(intentId, true);
	if (!intentResponse.isSome) {
		throw new Error(`Intent ${intentId} not found`);
	}

	const schemaIds = intentResponse.unwrap().schemaIds;
	if (!schemaIds.isSome || schemaIds.unwrap().length === 0) {
		return false;
	}

	const ids = schemaIds.unwrap();
	const latestSchemaId = ids[ids.length - 1];
	const latestSchemaResponse = await api.call.schemasRuntimeApi.getSchemaById(latestSchemaId);
	if (!latestSchemaResponse.isSome) {
		return false;
	}

	const latestSchema = latestSchemaResponse.unwrap();
	const latestModelType = latestSchema.modelType.toString();
	const latestModel = latestSchema.model.toUtf8();
	const normalizeJsonString = (jsonString) => {
		const sortKeysDeep = (value) => {
			if (Array.isArray(value)) {
				return value.map(sortKeysDeep);
			}
			if (value && typeof value === "object") {
				return Object.keys(value)
					.sort()
					.reduce((acc, key) => {
						acc[key] = sortKeysDeep(value[key]);
						return acc;
					}, {});
			}
			return value;
		};

		return JSON.stringify(sortKeysDeep(JSON.parse(jsonString)));
	};

	const normalizedLatestModel = normalizeJsonString(latestModel);
	const normalizedSchemaModel = normalizeJsonString(schemaDeploy.model);
	return latestModelType === schemaDeploy.model_type && normalizedLatestModel === normalizedSchemaModel ? latestSchemaId : false;
}

async function deploy(chainType, operationType) {
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
	
	const signerAccountKeys = getSignerAccountKeys();
	// Retrieve the current account nonce so we can increment it when submitting transactions
	let baseNonce = (await api.rpc.system.accountNextIndex(signerAccountKeys.address)).toNumber();

	const intentPromises = [];
	let intentNonceOffset = 0;
	for (const idx in INTENTS) {
		const intent = INTENTS[idx];
		const nonce = baseNonce + intentNonceOffset;
		// check if intent already exists
		const intentId = await getIntentId(api, intent);
		if (intentId) {
			console.log(`Intent "${intent.name}" already exists with ID ${intentId}`);
			intentPromises[idx] = Promise.resolve(intentId);
			continue;
		}
		if (chainType === MAINNET) {
			// create proposal
			if (operationType === INTENT) {
			 	intentPromises[idx] = getIntentProposalTransaction(api, signerAccountKeys, nonce, intent);
				intentNonceOffset += 1;
			} else { 
				intentPromises[idx] = getIntentId(api, intent);
			}
		} else {
			// create directly via sudo
			intentPromises[idx] = getIntentSudoTransaction(api, signerAccountKeys, nonce, intent);
			intentNonceOffset += 1;
		 }
	}
	const intentResults = await Promise.all(intentPromises);
	const idMap = new Map(intentResults.map((result, index) => {
		const id = Array.isArray(result) ? `${result[0]}` : `${result}`;
		return [INTENTS[index].name, parseInt(id, 10)];
	}));
	console.log('Resolved intents: ', idMap);
	baseNonce = (await api.rpc.system.accountNextIndex(signerAccountKeys.address)).toNumber();
	
	const schemaPromises = [];
	let schemaNonceOffset = 0;
	for (const idx in SCHEMAS) {
	  const schema = SCHEMAS[idx];
	  const intentId = await getIntentId(api, { name: schema.intent_name });
	  if (!intentId) {
		  throw new Error(`Intent ID not found for schema with intent_name: ${schema.intent_name}`);
	  }
	  console.log(`Found intentId ${intentId} for "${schema.intent_name}"`);

	  const duplicateSchemaId = await hasDuplicateLatestSchema(api, schema, intentId);
	  if (duplicateSchemaId) {
		console.log(
			`Skipping schema publish for intent ${schema.intent_name}: latest published schema (${duplicateSchemaId}) has the same model and modelType`,
		);
		schemaPromises[idx] = Promise.resolve(duplicateSchemaId);
		continue;
	  }

	  const nonce = baseNonce + schemaNonceOffset;

	  if (chainType === MAINNET) {
	    // create proposal
		if (operationType === SCHEMA) {
			schemaPromises[idx] = getSchemaProposalTransaction(
				api,
				signerAccountKeys,
				nonce,
				schema,
				intentId,
			);
			schemaNonceOffset += 1;
		}
	  } else {
	     // create directly via sudo
	     schemaPromises[idx] = getSchemaSudoTransaction(
	       api,
	       signerAccountKeys,
	       nonce,
	       schema,
		   intentId,
	     );
		 schemaNonceOffset += 1;
	  }
	}
    const schemaResults = await Promise.all(schemaPromises);
	console.log('Resolved/created schemas: ', new Map(schemaResults.map((result, index) => {
		const id = Array.isArray(result) ? `${result[0]}` : `${result}`;
		return [SCHEMAS[index].intent_name, parseInt(id, 10)];
	})));
}

// Given a list of events, a section and a method,
// returns the first event with matching section and method.
const eventWithSectionAndMethod = (events, section, method) => {
	const evt = events.find(({ event }) => event.section === section && event.method === method);
	return evt?.event;
};

function getIntentProposalTransaction(api, signerAccountKeys, nonce, intentDeploy) {
	// Propose to create
	const promise = new Promise((resolve, reject) => {
		api.tx.schemas
			.proposeToCreateIntent(
				intentDeploy.payload_location,
				intentDeploy.settings,
				intentDeploy.name,
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
								`${intentDeploy.name}` +
								" intent proposed with id of " +
								id +
								" and hash of " +
								hash,
						);
						resolve([id, hash]);
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

function getSchemaProposalTransaction(api, signerAccountKeys, nonce, schemaDeploy, intentId) {
	// Propose to create
	const promise = new Promise((resolve, reject) => {
		api.tx.schemas
			.proposeToCreateSchemaV3(
				intentId,
				schemaDeploy.model,
				schemaDeploy.model_type,
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
								`${schemaDeploy.intent_name}` +
								" schema proposed with id of " +
								id +
								" and hash of " +
								hash,
						);
						resolve([id, hash]);
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

function getSchemaSudoTransaction(api, signerAccountKeys, nonce, schemaDeploy, intent_id) {
	// Create directly via sudo
	const tx = api.tx.schemas.createSchemaViaGovernanceV3(
		signerAccountKeys.address,
		intent_id,
		schemaDeploy.model,
		schemaDeploy.model_type,
	);
	const promise = new Promise((resolve, reject) => {
		api.tx.sudo
			.sudo(tx)
			.signAndSend(signerAccountKeys, { nonce }, ({ status, events, dispatchError }) => {
				if (dispatchError) {
					console.error("Dispatch ERROR: ", dispatchError.toHuman());
					reject(dispatchError.toHuman());
				} else if (status.isInBlock || status.isFinalized) {
					const evt = eventWithSectionAndMethod(events, "schemas", "SchemaCreated");
					if (evt) {
						const id = evt?.data[1];
						resolve(id);
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
						resolve(id);
					} else {
						const err = "IntentCreated event not found";
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
		console.log("Uploading Intents & schemas");
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
		let operationType = "ALL";
		if (chainType === MAINNET) { 
			if (args.length < 2) { 
				console.log(`For Mainnet you must specify the operation type: ${INTENT} or ${SCHEMA}`);
				process.exit(1);
			}
			operationType = args[1].toUpperCase().trim();
			if (operationType !== INTENT && operationType !== SCHEMA) {
				console.log(`For Mainnet you must specify the operation type: ${INTENT} or ${SCHEMA}`);
				process.exit(1);
			}
		}
		
		await deploy(chainType, operationType);
		process.exit(0);
	} catch (error) {
		console.error("Error:", error);
		process.exit(1);
	}
}

main();

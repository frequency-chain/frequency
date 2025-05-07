import ethUtil from 'ethereumjs-util';
import abi from 'ethereumjs-abi';
import {expect} from 'chai';

const typedData = {
	types: {
		EIP712Domain: [
			{ name: 'name', type: 'string' },
			{ name: 'version', type: 'string' },
			{ name: 'chainId', type: 'uint256' },
			{ name: 'verifyingContract', type: 'address' },
		],
		ItemAction : [
			{
				name: 'actionType',
				type: 'string'
			},
			{
				name: 'data',
				type: 'bytes'
			},
			{
				name: 'index',
				type: 'uint16'
			}
		],
		ItemizedSignaturePayloadV2: [
			{
				name: "schemaId",
				type: "uint32"
			},
			{
				name: "targetHash",
				type: "uint64"
			},
			{
				name: "expiration",
				type: "uint64"
			},
			{
				name: "actions",
				type: "ItemAction[]"
			},
		],
	},
	primaryType: 'ItemizedSignaturePayloadV2',
	domain: {
		name: 'Frequency',
		version: '1',
		chainId: 0x190f1b44,
		verifyingContract: '0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC',
	},
	message: {
		schemaId: 10,
		targetHash: 1982672367,
		expiration: 100,
		actions: [
			{
				actionType: "Add",
				data: "0x40a6836ea489047852d3f0297f8fe8ad6779793af4e9c6274c230c207b9b825026",
				index: 0,
			},
			{
				actionType: "Delete",
				data: "0x",
				index: 2,
			},
		]
	},
};

const types = typedData.types;

// Recursively finds all the dependencies of a type
function dependencies(primaryType, found = []) {
	if (found.includes(primaryType)) {
		return found;
	}
	if (types[primaryType] === undefined) {
		return found;
	}
	found.push(primaryType);
	for (let field of types[primaryType]) {
		for (let dep of dependencies(field.type, found)) {
			if (!found.includes(dep)) {
				found.push(dep);
			}
		}
	}
	return found;
}

function encodeType(primaryType) {
	if (primaryType === 'ItemizedSignaturePayloadV2')
		return 'ItemizedSignaturePayloadV2(uint32 schemaId,uint64 targetHash,uint64 expiration,ItemAction[] actions)ItemAction(string actionType,bytes data,uint16 index)';
	// Get dependencies primary first, then alphabetical
	let deps = dependencies(primaryType);
	deps = deps.filter(t => t != primaryType);
	deps = [primaryType].concat(deps.sort());

	// Format as a string with fields
	let result = '';
	for (let type of deps) {
		result += `${type}(${types[type].map(({ name, type }) => `${type} ${name}`).join(',')})`;
	}
	return result;
}

function typeHash(primaryType) {
	return ethUtil.keccakFromString(encodeType(primaryType), 256);
}

function encodeData(primaryType, data) {
	let encTypes = [];
	let encValues = [];

	// Add typehash
	encTypes.push('bytes32');
	encValues.push(typeHash(primaryType));

	// Add field contents
	for (let field of types[primaryType]) {
		let value = data[field.name];
		if (field.type === 'string') {
			encTypes.push('bytes32');
			value = ethUtil.keccakFromString(value, 256);
			encValues.push(value);
		} else if(field.type === 'bytes') {
			encTypes.push('bytes32');
			value = ethUtil.keccakFromHexString(value);
			encValues.push(value);
		} else if (types[field.type] !== undefined) {
			encTypes.push('bytes32');
			value = ethUtil.keccak256(encodeData(field.type, value));
			encValues.push(value);
		} else if (field.type.lastIndexOf(']') === field.type.length - 1) {
			if (field.type.indexOf("ItemAction") >= 0) {
				encTypes.push('bytes32');
              	const baseType = field.type.replace('[]','');
				const arrayValues = value.map(function (v) {
					return structHash(baseType, v)
				});
				value = ethUtil.keccak256(Buffer.concat(arrayValues));
			  	encValues.push(value);
			} else {
				encTypes.push('bytes32');
				const unpacked = abi.solidityHexValue(field.type, value);
				encValues.push(ethUtil.keccak256(unpacked));
			}
		} else {
			encTypes.push(field.type);
			encValues.push(value);
		}
	}

	return abi.rawEncode(encTypes, encValues);
}

function structHash(primaryType, data) {
	return ethUtil.keccak256(encodeData(primaryType, data));
}

function signHash() {
	return ethUtil.keccak256(
		Buffer.concat([
			Buffer.from('1901', 'hex'),
			structHash('EIP712Domain', typedData.domain),
			structHash(typedData.primaryType, typedData.message),
		]),
	);
}

const privateKey = ethUtil.toBuffer('0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133');
const address = ethUtil.privateToAddress(privateKey);
console.log(ethUtil.bufferToHex(address));
console.log(ethUtil.bufferToHex(ethUtil.privateToPublic(privateKey)));
const sig = ethUtil.ecsign(signHash(), privateKey);
const compactSig = ethUtil.toCompactSig(sig.v, sig.r, sig.s);
console.log(compactSig);
const rpcSig = ethUtil.toRpcSig(sig.v, sig.r, sig.s);
console.log(rpcSig);

expect(encodeType('ItemizedSignaturePayloadV2')).to.equal('ItemizedSignaturePayloadV2(uint32 schemaId,uint64 targetHash,uint64 expiration,ItemAction[] actions)ItemAction(string actionType,bytes data,uint16 index)');
// expect(ethUtil.bufferToHex(typeHash('PaginatedUpsertSignaturePayloadV2'))).to.equal(
// 	'0xdf3ad6d56232d8c168e82ea91402346761c03cd54e834411fbf596716cd2953f',
// );
expect(ethUtil.bufferToHex(encodeData(typedData.primaryType, typedData.message))).to.equal(
	'0xc4e5f09322afb594fcd1f593e7c94de678b5ae1d8b6d6977000455c1826c75ce000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000762d2def0000000000000000000000000000000000000000000000000000000000000064e75a5168336e69346a737d11a1cfe9397a1c154347e684157c3ad02533777085',
);
// expect(ethUtil.bufferToHex(structHash(typedData.primaryType, typedData.message))).to.equal(
// 	'0x63a221fbfd2eef7102024c876cb2234276e924c5423a079f971e9d69260ba53e',
// );
// expect(encodeType('EIP712Domain')).to.equal('EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)');
// expect(ethUtil.bufferToHex(typeHash('EIP712Domain'))).to.equal(
// 	'0x8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f',
// );
// expect(ethUtil.bufferToHex(encodeData('EIP712Domain', typedData.domain))).to.equal(
// 	'0x8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400fd9b3cb8d2777b277796da1ccb2f8be9fa13f289418f2246fa1ecb7e94a0226c5c89efdaa54c0f20c7adf612882df0950f5a951637e0307cdcb4c672f298b8bc600000000000000000000000000000000000000000000000000000000190f1b44000000000000000000000000cccccccccccccccccccccccccccccccccccccccc',
// );
// expect(ethUtil.bufferToHex(structHash('EIP712Domain', typedData.domain))).to.equal(
// 	'0x8d95594185f4f2b6272976bb28848c643dff3308f3472a3c409526955cca05ab',
// );
// expect(ethUtil.bufferToHex(signHash())).to.equal('0x771ea6c6ee05d2383b361fea5aca32fde64a5a6ca0ea3b83304c50712dc22bd3');
// expect(ethUtil.bufferToHex(address)).to.equal('0xf24ff3a9cf04c71dbc94d0b566f7a27b94566cac');
// expect(sig.v).to.equal(27);
// expect(ethUtil.bufferToHex(sig.r)).to.equal('0x8ef06371476991364255f0f2cff46a4d756a8326e80567c074c10ab9503eaa86');
// expect(ethUtil.bufferToHex(sig.s)).to.equal('0x6145265e572ed857e7e215744a96c744980fd9fa1646beeb9aa508f5aafa845d');
// expect(compactSig).to.equal('0x12c6dc188563450175d7d68418004af167a44a0242e59a9b7c4f7bf1df43a8ef00b902a7efa580f1292a471241c267df6350b975d4523d8367c6485cd84a30b8');
expect(rpcSig).to.equal('0x7efb5407412c745f40713ba0922e228bf5f2b628423817a7a333a36902df0df45ef4cfd3c309fcaf9c0fc72e91a96b5456740b345283fc4525f5b09802ad1c0d1c');

// CommonPrimitivesHandlesClaimHandlePayload: {
// 	baseHandle: 'Bytes',
// 		expiration: 'u32'
// },

// BYTES -> Uint8Array


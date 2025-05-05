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
		ClaimHandlePayload: [
			{ name: 'handle', type: 'string' },
			{ name: 'expiration', type: 'uint64' }
		],
	},
	primaryType: 'ClaimHandlePayload',
	domain: {
		name: 'Frequency',
		version: '1',
		chainId: 0x190f1b44,
		verifyingContract: '0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC',
	},
	message: {
		handle : 'testing',
		expiration: 100,
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
		if (field.type == 'string' || field.type == 'bytes') {
			encTypes.push('bytes32');
			value = ethUtil.keccakFromString(value, 256);
			encValues.push(value);
		} else if (types[field.type] !== undefined) {
			encTypes.push('bytes32');
			value = ethUtil.keccak256(encodeData(field.type, value));
			encValues.push(value);
		} else if (field.type.lastIndexOf(']') === field.type.length - 1) {
			throw 'TODO: Arrays currently unimplemented in encodeData';
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

expect(encodeType('ClaimHandlePayload')).to.equal('ClaimHandlePayload(string handle,uint64 expiration)');
expect(ethUtil.bufferToHex(typeHash('ClaimHandlePayload'))).to.equal(
	'0xe5957ebf4e1c5ee379c679548f5a79f8976b3195249f2b5ffb1e3ed86b552aea',
);
expect(ethUtil.bufferToHex(encodeData(typedData.primaryType, typedData.message))).to.equal(
	'0xe5957ebf4e1c5ee379c679548f5a79f8976b3195249f2b5ffb1e3ed86b552aea5f16f4c7f149ac4f9510d9cf8cf384038ad348b3bcdc01915f95de12df9d1b020000000000000000000000000000000000000000000000000000000000000064',
);
expect(ethUtil.bufferToHex(structHash(typedData.primaryType, typedData.message))).to.equal(
	'0xd57570b549a5aac80e99464b2e58b975b13c240d992c8e8b7d81d8f32b7a3ac6',
);
expect(encodeType('EIP712Domain')).to.equal('EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)');
expect(ethUtil.bufferToHex(typeHash('EIP712Domain'))).to.equal(
	'0x8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f',
);
expect(ethUtil.bufferToHex(encodeData('EIP712Domain', typedData.domain))).to.equal(
	'0x8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400fd9b3cb8d2777b277796da1ccb2f8be9fa13f289418f2246fa1ecb7e94a0226c5c89efdaa54c0f20c7adf612882df0950f5a951637e0307cdcb4c672f298b8bc600000000000000000000000000000000000000000000000000000000190f1b44000000000000000000000000cccccccccccccccccccccccccccccccccccccccc',
);
expect(ethUtil.bufferToHex(structHash('EIP712Domain', typedData.domain))).to.equal(
	'0x8d95594185f4f2b6272976bb28848c643dff3308f3472a3c409526955cca05ab',
);
expect(ethUtil.bufferToHex(signHash())).to.equal('0x0af7dbd6bb58624546312e0512fa1e0e1eda3a15a9f36db56050a7b9a52747cb');
expect(ethUtil.bufferToHex(address)).to.equal('0xf24ff3a9cf04c71dbc94d0b566f7a27b94566cac');
expect(sig.v).to.equal(27);
expect(ethUtil.bufferToHex(sig.r)).to.equal('0x12c6dc188563450175d7d68418004af167a44a0242e59a9b7c4f7bf1df43a8ef');
expect(ethUtil.bufferToHex(sig.s)).to.equal('0x00b902a7efa580f1292a471241c267df6350b975d4523d8367c6485cd84a30b8');
expect(compactSig).to.equal('0x12c6dc188563450175d7d68418004af167a44a0242e59a9b7c4f7bf1df43a8ef00b902a7efa580f1292a471241c267df6350b975d4523d8367c6485cd84a30b8');
expect(rpcSig).to.equal('0x12c6dc188563450175d7d68418004af167a44a0242e59a9b7c4f7bf1df43a8ef00b902a7efa580f1292a471241c267df6350b975d4523d8367c6485cd84a30b81b');

console.log(ethUtil.bufferToHex(ethUtil.keccakFromString(typedData.message.handle)));
console.log(ethUtil.bufferToHex(ethUtil.keccakFromString('')))
// CommonPrimitivesHandlesClaimHandlePayload: {
// 	baseHandle: 'Bytes',
// 		expiration: 'u32'
// },

// BYTES -> Uint8Array


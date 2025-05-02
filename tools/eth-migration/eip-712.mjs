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
		HandlePayload: [
			{ name: 'handle', type: 'string' },
			{ name: 'expiration', type: 'uint64' }
		],
	},
	primaryType: 'HandlePayload',
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

expect(encodeType('HandlePayload')).to.equal('HandlePayload(string handle,uint64 expiration)');
expect(ethUtil.bufferToHex(typeHash('HandlePayload'))).to.equal(
	'0x4afae8095462377dc2c982219ff9adf392a625301e726c464bdb2be7ecbcb623',
);
expect(ethUtil.bufferToHex(encodeData(typedData.primaryType, typedData.message))).to.equal(
	'0x4afae8095462377dc2c982219ff9adf392a625301e726c464bdb2be7ecbcb6235f16f4c7f149ac4f9510d9cf8cf384038ad348b3bcdc01915f95de12df9d1b020000000000000000000000000000000000000000000000000000000000000064',
);
expect(ethUtil.bufferToHex(structHash(typedData.primaryType, typedData.message))).to.equal(
	'0x1e238a8f1e6502b4718d2ca0890bf2c6da27ac3877d3068f7b642d3454ad7ad1',
);
expect(ethUtil.bufferToHex(structHash('EIP712Domain', typedData.domain))).to.equal(
	'0x8d95594185f4f2b6272976bb28848c643dff3308f3472a3c409526955cca05ab',
);
expect(ethUtil.bufferToHex(signHash())).to.equal('0xacc2c61517380b57822b81c564ea486d6463d3c7956df925c0a8da0e19bb8c22');
expect(ethUtil.bufferToHex(address)).to.equal('0xf24ff3a9cf04c71dbc94d0b566f7a27b94566cac');
expect(sig.v).to.equal(28);
expect(ethUtil.bufferToHex(sig.r)).to.equal('0x146a9f0deea81fff681ab62e19485b727d99f02c95f3f98aaf738c2ef3c9bcee');
expect(ethUtil.bufferToHex(sig.s)).to.equal('0x16b925421e71d35cdca412f4d5cddd2d3f34845bd47de0c4fa6d5291f0e770e2');
expect(compactSig).to.equal('0x146a9f0deea81fff681ab62e19485b727d99f02c95f3f98aaf738c2ef3c9bcee96b925421e71d35cdca412f4d5cddd2d3f34845bd47de0c4fa6d5291f0e770e2');
expect(rpcSig).to.equal('0x146a9f0deea81fff681ab62e19485b727d99f02c95f3f98aaf738c2ef3c9bcee16b925421e71d35cdca412f4d5cddd2d3f34845bd47de0c4fa6d5291f0e770e21c');

console.log(ethUtil.bufferToHex(ethUtil.keccakFromString(typedData.message.handle)));
console.log(ethUtil.bufferToHex(ethUtil.keccakFromString('')))
// CommonPrimitivesHandlesClaimHandlePayload: {
// 	baseHandle: 'Bytes',
// 		expiration: 'u32'
// },

// BYTES -> Uint8Array


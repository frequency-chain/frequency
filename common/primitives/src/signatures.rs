#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "serde")]
use frame_support::{Deserialize, Serialize};
use frame_support::{
	__private::{codec, RuntimeDebug},
	pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo},
};
use parity_scale_codec::{alloc::string::ToString, DecodeWithMemTracking};
use sp_core::{
	crypto,
	crypto::{AccountId32, FromEntropy},
	ecdsa, ed25519,
	hexdisplay::HexDisplay,
	sr25519, ByteArray, H256,
};
use sp_runtime::{
	traits,
	traits::{Lazy, Verify},
	MultiSignature,
};
extern crate alloc;

/// Ethereum message prefix eip-191
const ETHEREUM_MESSAGE_PREFIX: &[u8; 26] = b"\x19Ethereum Signed Message:\n";

/// A trait that allows mapping of raw bytes to AccountIds
pub trait AccountAddressMapper<AccountId> {
	/// mapping to the desired address
	fn to_account_id(public_key_or_address: &[u8]) -> AccountId;

	/// mapping to bytes of a public key or an address
	fn to_bytes32(public_key_or_address: &[u8]) -> [u8; 32];

	/// reverses an accountId to it's 20 byte ethereum address
	fn to_ethereum_address(account_id: AccountId) -> [u8; 20];
}

/// converting raw address bytes to 32 bytes Ethereum compatible addresses
pub struct EthereumAddressMapper;

impl AccountAddressMapper<AccountId32> for EthereumAddressMapper {
	fn to_account_id(public_key_or_address: &[u8]) -> AccountId32 {
		Self::to_bytes32(public_key_or_address).into()
	}

	/// In this function we are trying to convert different types of valid identifiers to valid
	/// Substrate supported 32 bytes AccountIds
	/// ref: <https://github.com/paritytech/polkadot-sdk/blob/79b28b3185d01f2e43e098b1f57372ed9df64adf/substrate/frame/revive/src/address.rs#L84-L90>
	/// This function have 4 types of valid inputs
	/// 1. 20 byte ETH address which gets appended with 12 bytes of 0xEE
	/// 2. 32 byte address is returned unchanged
	/// 3. 64 bytes Secp256k1 public key is converted to ETH address based on <https://asecuritysite.com/encryption/ethadd> and appended 12 bytes of 0xEE
	/// 4. 65 bytes Secp256k1 public key is also converted to ETH address after skipping first byte and appended 12 bytes of 0xEE
	///    Anything else is invalid and would return default (all zeros) 32 bytes.
	fn to_bytes32(public_key_or_address: &[u8]) -> [u8; 32] {
		let mut hashed = [0u8; 32];
		match public_key_or_address.len() {
			20 => {
				hashed[..20].copy_from_slice(public_key_or_address);
			},
			32 => {
				hashed[..].copy_from_slice(public_key_or_address);
				return hashed;
			},
			64 => {
				let hashed_full = sp_io::hashing::keccak_256(public_key_or_address);
				// Copy bytes 12..32 (20 bytes) from hashed_full to the beginning of hashed
				hashed[..20].copy_from_slice(&hashed_full[12..]);
			},
			65 => {
				let hashed_full = sp_io::hashing::keccak_256(&public_key_or_address[1..]);
				// Copy bytes 12..32 (20 bytes) from hashed_full to the beginning of hashed
				hashed[..20].copy_from_slice(&hashed_full[12..]);
			},
			_ => {
				log::error!("Invalid public key size provided for {:?}", public_key_or_address);
				return [0u8; 32];
			},
		};

		// Fill the rest (12 bytes) with 0xEE
		hashed[20..].fill(0xEE);
		hashed
	}

	fn to_ethereum_address(account_id: AccountId32) -> [u8; 20] {
		let mut eth_address = [0u8; 20];
		if account_id.as_slice()[20..] == *[0xEE; 12].as_slice() {
			eth_address[..].copy_from_slice(&account_id.as_slice()[0..20]);
		}
		eth_address
	}
}

/// Signature verify that can work with any known signature types.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(
	Eq,
	PartialEq,
	Clone,
	Encode,
	Decode,
	DecodeWithMemTracking,
	MaxEncodedLen,
	RuntimeDebug,
	TypeInfo,
)]
pub enum UnifiedSignature {
	/// An Ed25519 signature.
	Ed25519(ed25519::Signature),
	/// An Sr25519 signature.
	Sr25519(sr25519::Signature),
	/// An ECDSA/SECP256k1 signature compatible with Ethereum
	Ecdsa(ecdsa::Signature),
}

impl From<ed25519::Signature> for UnifiedSignature {
	fn from(x: ed25519::Signature) -> Self {
		Self::Ed25519(x)
	}
}

impl TryFrom<UnifiedSignature> for ed25519::Signature {
	type Error = ();
	fn try_from(m: UnifiedSignature) -> Result<Self, Self::Error> {
		if let UnifiedSignature::Ed25519(x) = m {
			Ok(x)
		} else {
			Err(())
		}
	}
}

impl From<sr25519::Signature> for UnifiedSignature {
	fn from(x: sr25519::Signature) -> Self {
		Self::Sr25519(x)
	}
}

impl TryFrom<UnifiedSignature> for sr25519::Signature {
	type Error = ();
	fn try_from(m: UnifiedSignature) -> Result<Self, Self::Error> {
		if let UnifiedSignature::Sr25519(x) = m {
			Ok(x)
		} else {
			Err(())
		}
	}
}

impl From<ecdsa::Signature> for UnifiedSignature {
	fn from(x: ecdsa::Signature) -> Self {
		Self::Ecdsa(x)
	}
}

impl TryFrom<UnifiedSignature> for ecdsa::Signature {
	type Error = ();
	fn try_from(m: UnifiedSignature) -> Result<Self, Self::Error> {
		if let UnifiedSignature::Ecdsa(x) = m {
			Ok(x)
		} else {
			Err(())
		}
	}
}

impl Verify for UnifiedSignature {
	type Signer = UnifiedSigner;
	fn verify<L: Lazy<[u8]>>(&self, msg: L, signer: &AccountId32) -> bool {
		match (self, signer) {
			(Self::Ed25519(ref sig), who) => match ed25519::Public::from_slice(who.as_ref()) {
				Ok(signer) => sig.verify(msg, &signer),
				Err(()) => false,
			},
			(Self::Sr25519(ref sig), who) => match sr25519::Public::from_slice(who.as_ref()) {
				Ok(signer) => sig.verify(msg, &signer),
				Err(()) => false,
			},
			(Self::Ecdsa(ref sig), who) => check_ethereum_signature(sig, msg, who),
		}
	}
}

/// Public key for any known crypto algorithm.
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum UnifiedSigner {
	/// An Ed25519 identity.
	Ed25519(ed25519::Public),
	/// An Sr25519 identity.
	Sr25519(sr25519::Public),
	/// An SECP256k1/ECDSA identity (12 bytes of zeros + 20 bytes of ethereum address).
	Ecdsa(ecdsa::Public),
}

impl FromEntropy for UnifiedSigner {
	fn from_entropy(input: &mut impl codec::Input) -> Result<Self, codec::Error> {
		Ok(match input.read_byte()? % 3 {
			0 => Self::Ed25519(FromEntropy::from_entropy(input)?),
			1 => Self::Sr25519(FromEntropy::from_entropy(input)?),
			2.. => Self::Ecdsa(FromEntropy::from_entropy(input)?),
		})
	}
}

/// NOTE: This implementations is required by `SimpleAddressDeterminer`,
/// we convert the hash into some AccountId, it's fine to use any scheme.
impl<T: Into<H256>> crypto::UncheckedFrom<T> for UnifiedSigner {
	fn unchecked_from(x: T) -> Self {
		ed25519::Public::unchecked_from(x.into()).into()
	}
}

impl AsRef<[u8]> for UnifiedSigner {
	fn as_ref(&self) -> &[u8] {
		match *self {
			Self::Ed25519(ref who) => who.as_ref(),
			Self::Sr25519(ref who) => who.as_ref(),
			Self::Ecdsa(ref who) => who.as_ref(),
		}
	}
}

impl traits::IdentifyAccount for UnifiedSigner {
	type AccountId = AccountId32;
	fn into_account(self) -> AccountId32 {
		match self {
			Self::Ed25519(who) => <[u8; 32]>::from(who).into(),
			Self::Sr25519(who) => <[u8; 32]>::from(who).into(),
			Self::Ecdsa(who) => {
				let decompressed_result = libsecp256k1::PublicKey::parse_slice(
					who.as_ref(),
					Some(libsecp256k1::PublicKeyFormat::Compressed),
				);
				match decompressed_result {
					Ok(public_key) => {
						// calculating ethereum address compatible with `pallet-revive`
						let decompressed = public_key.serialize();
						EthereumAddressMapper::to_account_id(&decompressed)
					},
					Err(_) => {
						log::error!("Invalid compressed public key provided");
						AccountId32::new([0u8; 32])
					},
				}
			},
		}
	}
}

impl From<ed25519::Public> for UnifiedSigner {
	fn from(x: ed25519::Public) -> Self {
		Self::Ed25519(x)
	}
}

impl TryFrom<UnifiedSigner> for ed25519::Public {
	type Error = ();
	fn try_from(m: UnifiedSigner) -> Result<Self, Self::Error> {
		if let UnifiedSigner::Ed25519(x) = m {
			Ok(x)
		} else {
			Err(())
		}
	}
}

impl From<sr25519::Public> for UnifiedSigner {
	fn from(x: sr25519::Public) -> Self {
		Self::Sr25519(x)
	}
}

impl TryFrom<UnifiedSigner> for sr25519::Public {
	type Error = ();
	fn try_from(m: UnifiedSigner) -> Result<Self, Self::Error> {
		if let UnifiedSigner::Sr25519(x) = m {
			Ok(x)
		} else {
			Err(())
		}
	}
}

impl From<ecdsa::Public> for UnifiedSigner {
	fn from(x: ecdsa::Public) -> Self {
		Self::Ecdsa(x)
	}
}

impl TryFrom<UnifiedSigner> for ecdsa::Public {
	type Error = ();
	fn try_from(m: UnifiedSigner) -> Result<Self, Self::Error> {
		if let UnifiedSigner::Ecdsa(x) = m {
			Ok(x)
		} else {
			Err(())
		}
	}
}

#[cfg(feature = "std")]
impl std::fmt::Display for UnifiedSigner {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
		match *self {
			Self::Ed25519(ref who) => write!(fmt, "ed25519: {}", who),
			Self::Sr25519(ref who) => write!(fmt, "sr25519: {}", who),
			Self::Ecdsa(ref who) => write!(fmt, "ecdsa: {}", who),
		}
	}
}

impl Into<UnifiedSignature> for MultiSignature {
	fn into(self: MultiSignature) -> UnifiedSignature {
		match self {
			MultiSignature::Ed25519(who) => UnifiedSignature::Ed25519(who),
			MultiSignature::Sr25519(who) => UnifiedSignature::Sr25519(who),
			MultiSignature::Ecdsa(who) => UnifiedSignature::Ecdsa(who),
		}
	}
}

fn check_secp256k1_signature(signature: &[u8; 65], msg: &[u8; 32], signer: &AccountId32) -> bool {
	match sp_io::crypto::secp256k1_ecdsa_recover(signature, msg) {
		Ok(pubkey) => {
			let hashed = EthereumAddressMapper::to_bytes32(&pubkey);
			log::debug!(target:"ETHEREUM", "eth hashed={:?} signer={:?}",
				HexDisplay::from(&hashed),HexDisplay::from(<dyn AsRef<[u8; 32]>>::as_ref(signer)),
			);
			&hashed == <dyn AsRef<[u8; 32]>>::as_ref(signer)
		},
		_ => false,
	}
}

fn eth_message_hash(message: &[u8]) -> [u8; 32] {
	let only_len = (message.len() as u32).to_string().into_bytes();
	let concatenated = [ETHEREUM_MESSAGE_PREFIX.as_slice(), only_len.as_slice(), message].concat();
	log::debug!(target:"ETHEREUM", "prefixed {:?}",concatenated);
	sp_io::hashing::keccak_256(concatenated.as_slice())
}

fn check_ethereum_signature<L: Lazy<[u8]>>(
	signature: &ecdsa::Signature,
	mut msg: L,
	signer: &AccountId32,
) -> bool {
	let verify_signature = |signature: &[u8; 65], payload: &[u8; 32], signer: &AccountId32| {
		check_secp256k1_signature(signature, payload, signer)
	};

	// signature of ethereum prefixed message eip-191
	let message_prefixed = eth_message_hash(msg.get());
	if verify_signature(signature.as_ref(), &message_prefixed, signer) {
		return true
	}

	// PolkadotJs raw payload signatures
	// or Ethereum based EIP-712 compatible signatures
	let hashed = sp_io::hashing::keccak_256(msg.get());
	verify_signature(signature.as_ref(), &hashed, signer)
}

#[cfg(test)]
mod tests {
	use crate::{
		handles::ClaimHandlePayload,
		signatures::{UnifiedSignature, UnifiedSigner},
		utils::EIP712Encode,
	};
	use impl_serde::serialize::from_hex;
	use sp_core::{ecdsa, Pair};
	use sp_runtime::{
		traits::{IdentifyAccount, Verify},
		AccountId32,
	};

	use super::{AccountAddressMapper, EthereumAddressMapper};

	#[test]
	fn polkadot_ecdsa_should_not_work_due_to_using_wrong_hash() {
		let msg = &b"test-message"[..];
		let (pair, _) = ecdsa::Pair::generate();

		let signature = pair.sign(msg);
		let unified_sig = UnifiedSignature::from(signature);
		let unified_signer = UnifiedSigner::from(pair.public());
		assert!(!unified_sig.verify(msg, &unified_signer.into_account()));
	}

	#[test]
	fn ethereum_prefixed_eip191_signatures_should_work() {
		// payload is random and the signature is generated over that payload by a standard EIP-191 signer
		let payload = b"test eip-191 message payload";
		let signature_raw = from_hex("0x276dcc9c69da24dd8441ba3acc9b60d8aae0cb39f0bc5ad92c723a31bf11575031d860978280191a0a97a1f74336ca0c79a8b1b3aab013fb58a27f113b73b2081b").expect("Should convert");
		let unified_signature = UnifiedSignature::from(ecdsa::Signature::from_raw(
			signature_raw.try_into().expect("should convert"),
		));

		let public_key = ecdsa::Public::from_raw(
			from_hex("0x03be5b145e12c5fb95151374ed919eb445ade57637d729dd2d73bf161d4bc10329")
				.expect("should convert")
				.try_into()
				.expect("invalid size"),
		);
		let unified_signer = UnifiedSigner::from(public_key);
		assert!(unified_signature.verify(&payload[..], &unified_signer.into_account()));
	}

	#[test]
	fn ethereum_raw_signatures_should_work() {
		// payload is random and the signature is generated over that payload by PolkadotJs and ethereum keypair
		let payload = from_hex("0x0a0300e659a7a1628cdd93febc04a4e0646ea20e9f5f0ce097d9a05290d4a9e054df4e028c7d0a3500000000830000000100000026c1147602cf6557f4e0068a78cd4b22b6f6b03e106d05618cde8537e4ffe454b63f7774106903a22684c02eeebe2fdc903ac945bf25962fd9d05e7e0ddfb44f00").expect("Should convert");
		let signature_raw = from_hex("0xd740c8294967b36236c5e05861a55bad75d0866c4a6f63d4918a39769a9582b872299a3411cc0f31b5f631261d669fc21ce427ee23999a91df5f0e74dfbbfc6c00").expect("Should convert");
		let unified_signature = UnifiedSignature::from(ecdsa::Signature::from_raw(
			signature_raw.try_into().expect("should convert"),
		));

		let public_key = ecdsa::Public::from_raw(
			from_hex("0x025b107c7f38d5ac7d618e626f9fa57eec683adf373b1352cd20e5e5c684747079")
				.expect("should convert")
				.try_into()
				.expect("invalid size"),
		);
		let unified_signer = UnifiedSigner::from(public_key);
		assert!(unified_signature.verify(&payload[..], &unified_signer.into_account()));
	}

	#[test]
	fn ethereum_eip712_signatures_for_claim_handle_payload_should_work() {
		// 0x5f16f4c7f149ac4f9510d9cf8cf384038ad348b3bcdc01915f95de12df9d1b02 testing
		// 0x0000000000000000000000000000000000000000000000000000000000000064 100
		// let encoded_payload = from_hex("0x5f16f4c7f149ac4f9510d9cf8cf384038ad348b3bcdc01915f95de12df9d1b020000000000000000000000000000000000000000000000000000000000000064").expect("Should convert");
		let payload = ClaimHandlePayload { base_handle: b"testing".to_vec(), expiration: 100u32 };
		let encoded_payload = payload.encode_eip_712();

		let signature_raw = from_hex("0x12c6dc188563450175d7d68418004af167a44a0242e59a9b7c4f7bf1df43a8ef00b902a7efa580f1292a471241c267df6350b975d4523d8367c6485cd84a30b81b").expect("Should convert");
		let unified_signature = UnifiedSignature::from(ecdsa::Signature::from_raw(
			signature_raw.try_into().expect("should convert"),
		));

		// 0x509540919faacf9ab52146c9aa40db68172d83777250b28e4679176e49ccdd9fa213197dc0666e85529d6c9dda579c1295d61c417f01505765481e89a4016f02
		let public_key = ecdsa::Public::from_raw(
			from_hex("0x02509540919faacf9ab52146c9aa40db68172d83777250b28e4679176e49ccdd9f")
				.expect("should convert")
				.try_into()
				.expect("invalid size"),
		);
		let unified_signer = UnifiedSigner::from(public_key);
		assert!(unified_signature.verify(&encoded_payload[..], &unified_signer.into_account()));
	}

	#[test]
	fn ethereum_invalid_signatures_should_fail() {
		let payload = from_hex("0x0a0300e659a7a1628cdd93febc04a4e0646ea20e9f5f0ce097d9a05290d4a9e054df4e028c7d0a3500000000830000000100000026c1147602cf6557f4e0068a78cd4b22b6f6b03e106d05618cde8537e4ffe4548de1bcb12a1d42e58b218a7abb03cb629111625cf3449640d837c5aa98b87d8e00").expect("Should convert");
		let signature_raw = from_hex("0x9633e747bcd951bdb9d98ff84c65562e1f62bd059c578a942859e1695f2472aa0dbaab48c28f6dbc795baa73c27252d97e8dc2170fd7d69694d5cd1863fb968c01").expect("Should convert");
		let unified_signature = UnifiedSignature::from(ecdsa::Signature::from_raw(
			signature_raw.try_into().expect("should convert"),
		));

		let public_key = ecdsa::Public::from_raw(
			from_hex("0x025b107c7f38d5ac7d618e626f9fa57eec683adf373b1352cd20e5e5c684747079")
				.expect("should convert")
				.try_into()
				.expect("invalid size"),
		);
		let unified_signer = UnifiedSigner::from(public_key);
		assert!(!unified_signature.verify(&payload[..], &unified_signer.into_account()));
	}

	#[test]
	fn ethereum_address_mapper_should_work_as_expected_for_eth_20_bytes_addresses() {
		// arrange
		let eth = from_hex("0x1111111111111111111111111111111111111111").expect("should work");

		// act
		let account_id = EthereumAddressMapper::to_account_id(&eth);
		let bytes = EthereumAddressMapper::to_bytes32(&eth);

		// assert
		let expected_address =
			from_hex("0x1111111111111111111111111111111111111111eeeeeeeeeeeeeeeeeeeeeeee")
				.expect("should be hex");
		assert_eq!(account_id, AccountId32::new(expected_address.clone().try_into().unwrap()));
		assert_eq!(bytes.to_vec(), expected_address);
	}

	#[test]
	fn ethereum_address_mapper_should_return_the_same_value_for_32_byte_addresses() {
		// arrange
		let eth = from_hex("0x1111111111111111111111111111111111111111111111111111111111111111")
			.expect("should work");

		// act
		let account_id = EthereumAddressMapper::to_account_id(&eth);
		let bytes = EthereumAddressMapper::to_bytes32(&eth);

		// assert
		assert_eq!(account_id, AccountId32::new(eth.clone().try_into().unwrap()));
		assert_eq!(bytes.to_vec(), eth);
	}

	#[test]
	fn ethereum_address_mapper_should_return_the_ethereum_address_with_suffixes_for_64_byte_public_keys(
	) {
		// arrange
		let public_key= from_hex("0x15b5e4aeac2086ee96ab2292ee2720da0b2d3c43b5c699ccdbfd38387e2f71dc167075a80a32fe2c78d7d8780ef1b2095810f12001fa2fcedcd1ffb0aa2ee2c7").expect("should work");

		// act
		let account_id = EthereumAddressMapper::to_account_id(&public_key);
		let bytes = EthereumAddressMapper::to_bytes32(&public_key);

		// assert
		// 0x917B536617B0A42B2ABE85AC88788825F29F0B29 is eth address associated with above public_key
		let expected_address =
			from_hex("0x917B536617B0A42B2ABE85AC88788825F29F0B29eeeeeeeeeeeeeeeeeeeeeeee")
				.expect("should be hex");
		assert_eq!(account_id, AccountId32::new(expected_address.clone().try_into().unwrap()));
		assert_eq!(bytes.to_vec(), expected_address);
	}

	#[test]
	fn ethereum_address_mapper_should_return_the_ethereum_address_with_suffixes_for_65_byte_public_keys(
	) {
		// arrange
		let public_key= from_hex("0x0415b5e4aeac2086ee96ab2292ee2720da0b2d3c43b5c699ccdbfd38387e2f71dc167075a80a32fe2c78d7d8780ef1b2095810f12001fa2fcedcd1ffb0aa2ee2c7").expect("should work");

		// act
		let account_id = EthereumAddressMapper::to_account_id(&public_key);
		let bytes = EthereumAddressMapper::to_bytes32(&public_key);

		// assert
		// 0x917B536617B0A42B2ABE85AC88788825F29F0B29 is eth address associated with above public_key
		let expected_address =
			from_hex("0x917B536617B0A42B2ABE85AC88788825F29F0B29eeeeeeeeeeeeeeeeeeeeeeee")
				.expect("should be hex");
		assert_eq!(account_id, AccountId32::new(expected_address.clone().try_into().unwrap()));
		assert_eq!(bytes.to_vec(), expected_address);
	}

	#[test]
	fn ethereum_address_mapper_should_return_the_default_zero_values_for_any_invalid_length() {
		// arrange
		let public_key = from_hex(
			"0x010415b5e4aeac2086ee96ab2292ee2720da0b2d3c43b5c699ccdbfd38387e2f71dc167075a801",
		)
		.expect("should work");

		// act
		let account_id = EthereumAddressMapper::to_account_id(&public_key);
		let bytes = EthereumAddressMapper::to_bytes32(&public_key);

		// assert
		let expected_address = vec![0u8; 32]; // zero default values
		assert_eq!(account_id, AccountId32::new(expected_address.clone().try_into().unwrap()));
		assert_eq!(bytes.to_vec(), expected_address);
	}
}

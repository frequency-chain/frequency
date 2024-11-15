#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "serde")]
use frame_support::{Deserialize, Serialize};
use frame_support::{
	__private::{codec, RuntimeDebug},
	pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo},
};
use scale_info::prelude::format;
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

/// Signature verify that can work with any known signature types.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, MaxEncodedLen, RuntimeDebug, TypeInfo)]
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
						// calculating ethereum address prefixed with zeros
						let decompressed = public_key.serialize();
						let mut hashed = sp_io::hashing::keccak_256(&decompressed[1..65]);
						hashed[..12].fill(0);
						hashed.into()
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
	match sp_io::crypto::secp256k1_ecdsa_recover(signature, &msg) {
		Ok(pubkey) => {
			let mut hashed = sp_io::hashing::keccak_256(pubkey.as_ref());
			hashed[..12].fill(0);
			log::debug!(target:"ETHEREUM", "eth hashed={:?} signer={:?}",
				HexDisplay::from(&hashed),HexDisplay::from(<dyn AsRef<[u8; 32]>>::as_ref(signer)),
			);
			&hashed == <dyn AsRef<[u8; 32]>>::as_ref(signer)
		},
		_ => false,
	}
}

fn eth_message_hash(message: &str) -> [u8; 32] {
	let prefixed = format!("{}{}{}", "\x19Ethereum Signed Message:\n", message.len(), message);
	log::debug!(target:"ETHEREUM", "prefixed {:?}",prefixed);
	sp_io::hashing::keccak_256(prefixed.as_bytes())
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
	let message_prefixed = eth_message_hash(&format!("0x{:?}", HexDisplay::from(&msg.get())));
	if verify_signature(&signature.as_ref(), &message_prefixed, signer) {
		return true
	}

	// signature of raw payload, compatible with polkadotJs signatures
	let hashed = sp_io::hashing::keccak_256(&msg.get());
	if verify_signature(signature.as_ref(), &hashed, signer) {
		return true
	}

	// frequency wrapped for Metamask compatibility
	let frequency_wrapped =
		eth_message_hash(&format!("<Frequency>0x{:?}</Frequency>", HexDisplay::from(&msg.get())));
	verify_signature(&signature.as_ref(), &frequency_wrapped, signer)
}

#[cfg(test)]
mod tests {
	use crate::signatures::{UnifiedSignature, UnifiedSigner};
	use impl_serde::serialize::from_hex;
	use sp_core::{ecdsa, Pair};
	use sp_runtime::traits::{IdentifyAccount, Verify};

	#[test]
	fn polkadot_ecdsa_should_not_work_due_to_using_wrong_hash() {
		let msg = &b"test-message"[..];
		let (pair, _) = ecdsa::Pair::generate();

		let signature = pair.sign(&msg);
		let unified_sig = UnifiedSignature::from(signature);
		let unified_signer = UnifiedSigner::from(pair.public());
		assert_eq!(unified_sig.verify(msg, &unified_signer.into_account()), false);
	}

	#[test]
	fn ethereum_prefixed_eip191_signatures_should_work() {
		// payload is random and the signature is generated over that payload by a standard EIP-191 signer
		let payload = from_hex("0x0a0300e659a7a1628cdd93febc04a4e0646ea20e9f5f0ce097d9a05290d4a9e054df4e028c7d0a3500000000830000000100000026c1147602cf6557f4e0068a78cd4b22b6f6b03e106d05618cde8537e4ffe454c1f285c69f563934857a63463571d57723fbad6ac7de44611ed674f02c04c2ae00").expect("Should convert");
		let signature_raw = from_hex("0x056ca64d31251a1f20733ce2a741e2963c87a9674a35f8619b6b97210ae8c8b54c2853da1b943dd95ac3b893b37f69ca7dc38c13f8ec92a235b00ae03426505900").expect("Should convert");
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
	fn ethereum_custom_wrapped_signatures_should_work() {
		// payload is random and the signature is generated over that payload by Metamask signer
		let payload = from_hex("0x0a0300e659a7a1628cdd93febc04a4e0646ea20e9f5f0ce097d9a05290d4a9e054df4e028c7d0a3500000000830000000100000026c1147602cf6557f4e0068a78cd4b22b6f6b03e106d05618cde8537e4ffe4548de1bcb12a1d42e58b218a7abb03cb629111625cf3449640d837c5aa98b87d8e00").expect("Should convert");
		let signature_raw = from_hex("0x9633e747bcd951bdb9d98ff84c65562e1f62bd059c578a942859e1695f2472aa0dbaab48c28f6dbc795baa73c27252d97e8dc2170fd7d69694d5cd1863fb968c00").expect("Should convert");
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
		assert_eq!(unified_signature.verify(&payload[..], &unified_signer.into_account()), false);
	}
}

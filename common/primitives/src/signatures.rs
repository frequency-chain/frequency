#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(feature = "serde")]
use frame_support::{Deserialize, Serialize};
use frame_support::{
	__private::{codec, RuntimeDebug},
	pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo},
};
use sp_core::{
	crypto,
	crypto::{AccountId32, FromEntropy},
	ecdsa, ed25519,
	hexdisplay::HexDisplay,
	sr25519, ByteArray, H256,
};
use scale_info::prelude::format;
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
	/// An ECDSA/SECP256k1 signature.
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
	fn verify<L: Lazy<[u8]>>(&self, mut msg: L, signer: &AccountId32) -> bool {
		match (self, signer) {
			(Self::Ed25519(ref sig), who) => match ed25519::Public::from_slice(who.as_ref()) {
				Ok(signer) => sig.verify(msg, &signer),
				Err(()) => false,
			},
			(Self::Sr25519(ref sig), who) => match sr25519::Public::from_slice(who.as_ref()) {
				Ok(signer) => sig.verify(msg, &signer),
				Err(()) => false,
			},
			(Self::Ecdsa(ref sig), who) => {
				log::info!(target:"ETHEREUM", "inside ecdsa signature verifier 0x{:?}",HexDisplay::from(&msg.get()));
				let m = eth_message(&format!("<Frequency>0x{:?}</Frequency>", HexDisplay::from(&msg.get())));
				log::info!(target:"ETHEREUM", "prefixed hashed 0x{:?}",HexDisplay::from(&m));
				match sp_io::crypto::secp256k1_ecdsa_recover(sig.as_ref(), &m) {
					Ok(pubkey) => {
						let mut hashed = sp_io::hashing::keccak_256(pubkey.as_ref());
						hashed[..12].fill(0);
						log::info!(target:"ETHEREUM", "eth hashed={:?} who={:?}",
							HexDisplay::from(&hashed),HexDisplay::from(<dyn AsRef<[u8; 32]>>::as_ref(who)),
						);
						&hashed == <dyn AsRef<[u8; 32]>>::as_ref(who)
					},
					_ => false,
				}
			},
		}
	}
}
fn eth_message(message: &str) -> [u8; 32] {
	let prefixed = format!("{}{}{}", "\x19Ethereum Signed Message:\n", message.len(), message);
	log::info!(target:"ETHEREUM", "prefixed {:?}",prefixed);
	sp_io::hashing::keccak_256(prefixed.as_bytes())
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
				log::info!(target:"ETHEREUM", "inside ecdsa into_account");
				let decompressed = libsecp256k1::PublicKey::parse_slice(
					who.as_ref(),
					Some(libsecp256k1::PublicKeyFormat::Compressed),
				)
				.expect("Wrong compressed public key provided")
				.serialize();
				let mut m = [0u8; 64];
				m.copy_from_slice(&decompressed[1..65]);
				let mut hashed = sp_io::hashing::keccak_256(m.as_ref());
				hashed[..12].fill(0);
				hashed.into()
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

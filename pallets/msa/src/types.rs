//! Types for the MSA Pallet
#![cfg_attr(not(feature = "std"), no_std)]

use super::*;
use parity_scale_codec::{Decode, Encode};

use core::fmt::Debug;

pub use common_primitives::msa::{
	ApplicationIndex, Delegation, DelegatorId, KeyInfoResponse, MessageSourceId, ProviderId,
};
use common_primitives::node::BlockNumber;

use common_primitives::{
	signatures::{get_eip712_encoding_prefix, AccountAddressMapper, EthereumAddressMapper},
	utils::to_abi_compatible_number,
};
use scale_info::TypeInfo;
use sp_core::U256;

/// LogoCID type
pub type LogoCid<T> = BoundedVec<u8, <T as Config>::MaxLogoCidSize>;

/// A type definition for the payload for the following operation:
/// -  Adding an MSA key - `pallet_msa::add_public_key_to_msa`
#[derive(
	TypeInfo, RuntimeDebugNoBound, Clone, Decode, DecodeWithMemTracking, Encode, PartialEq, Eq,
)]
#[scale_info(skip_type_params(T))]
pub struct AddKeyData<T: Config> {
	/// Message Source Account identifier
	pub msa_id: MessageSourceId,
	/// The block number at which a signed proof of this payload expires.
	pub expiration: BlockNumberFor<T>,
	/// The public key to be added.
	pub new_public_key: T::AccountId,
}

impl<T: Config> EIP712Encode for AddKeyData<T> {
	fn encode_eip_712(&self, chain_id: u32) -> Box<[u8]> {
		lazy_static! {
			// signed payload
			static ref MAIN_TYPE_HASH: [u8; 32] = sp_io::hashing::keccak_256(
				b"AddKeyData(uint64 msaId,uint32 expiration,address newPublicKey)",
			);
		}
		// get prefix and domain separator
		let prefix_domain_separator: Box<[u8]> =
			get_eip712_encoding_prefix("0xcccccccccccccccccccccccccccccccccccccccc", chain_id);
		let coded_owner_msa_id = to_abi_compatible_number(self.msa_id);
		let expiration: U256 = self.expiration.into();
		let coded_expiration = to_abi_compatible_number(expiration.as_u128());
		let converted_public_key = T::ConvertIntoAccountId32::convert(self.new_public_key.clone());
		let mut zero_prefixed_address = [0u8; 32];
		zero_prefixed_address[12..]
			.copy_from_slice(&EthereumAddressMapper::to_ethereum_address(converted_public_key).0);
		let message = sp_io::hashing::keccak_256(
			&[
				MAIN_TYPE_HASH.as_slice(),
				&coded_owner_msa_id,
				&coded_expiration,
				&zero_prefixed_address,
			]
			.concat(),
		);
		let combined = [prefix_domain_separator.as_ref(), &message].concat();
		combined.into_boxed_slice()
	}
}

/// Type discriminator enum for signed payloads
#[derive(
	Encode, Decode, DecodeWithMemTracking, Clone, Copy, PartialEq, Eq, RuntimeDebugNoBound, TypeInfo,
)]
pub enum PayloadTypeDiscriminator {
	/// Unknown payload type
	Unknown,
	/// AuthorizedKeyData discriminator
	AuthorizedKeyData,
	/// RecoverCommitmentPayload discriminator
	RecoveryCommitmentPayload,
}

/// A type definition for the payload for authorizing a public key for the following operations:
/// -  Authorizing a token withdrawal to an address associated with a public key - `pallet_msa::withdraw_tokens`
#[derive(
	TypeInfo, RuntimeDebugNoBound, Clone, Decode, DecodeWithMemTracking, Encode, PartialEq, Eq,
)]
#[scale_info(skip_type_params(T))]
pub struct AuthorizedKeyData<T: Config> {
	/// type discriminator, because otherwise this struct has the same SCALE encoding as `AddKeyData`
	pub discriminant: PayloadTypeDiscriminator,
	/// Message Source Account identifier
	pub msa_id: MessageSourceId,
	/// The block number at which a signed proof of this payload expires.
	pub expiration: BlockNumberFor<T>,
	/// The public key to be added that is authorized as the target of the current operation
	pub authorized_public_key: T::AccountId,
}

impl<T: Config> EIP712Encode for AuthorizedKeyData<T> {
	fn encode_eip_712(&self, chain_id: u32) -> Box<[u8]> {
		lazy_static! {
			// signed payload
			static ref MAIN_TYPE_HASH: [u8; 32] = sp_io::hashing::keccak_256(
				b"AuthorizedKeyData(uint64 msaId,uint32 expiration,address authorizedPublicKey)",
			);
		}
		// get prefix and domain separator
		let prefix_domain_separator: Box<[u8]> =
			get_eip712_encoding_prefix("0xcccccccccccccccccccccccccccccccccccccccc", chain_id);
		let coded_owner_msa_id = to_abi_compatible_number(self.msa_id);
		let expiration: U256 = self.expiration.into();
		let coded_expiration = to_abi_compatible_number(expiration.as_u128());
		let converted_public_key =
			T::ConvertIntoAccountId32::convert(self.authorized_public_key.clone());
		let mut zero_prefixed_address = [0u8; 32];
		zero_prefixed_address[12..]
			.copy_from_slice(&EthereumAddressMapper::to_ethereum_address(converted_public_key).0);
		let message = sp_io::hashing::keccak_256(
			&[
				MAIN_TYPE_HASH.as_slice(),
				&coded_owner_msa_id,
				&coded_expiration,
				&zero_prefixed_address,
			]
			.concat(),
		);
		let combined = [prefix_domain_separator.as_ref(), &message].concat();
		combined.into_boxed_slice()
	}
}

/// Structure that is signed for granting permissions to a Provider
#[derive(TypeInfo, Clone, Debug, Decode, DecodeWithMemTracking, Encode, PartialEq, Eq)]
pub struct AddProvider {
	/// The provider being granted permissions
	pub authorized_msa_id: MessageSourceId,
	/// Schemas for which publishing grants are authorized.
	/// This is private intended for internal use only.
	pub intent_ids: Vec<IntentId>,
	/// The block number at which the proof for grant_delegation expires.
	pub expiration: BlockNumber,
}

impl EIP712Encode for AddProvider {
	fn encode_eip_712(&self, chain_id: u32) -> Box<[u8]> {
		lazy_static! {
			// signed payload
			static ref MAIN_TYPE_HASH: [u8; 32] = sp_io::hashing::keccak_256(
				b"AddProvider(uint64 authorizedMsaId,uint16[] intentIds,uint32 expiration)"
			);
		}
		// get prefix and domain separator
		let prefix_domain_separator: Box<[u8]> =
			get_eip712_encoding_prefix("0xcccccccccccccccccccccccccccccccccccccccc", chain_id);
		let coded_authorized_msa_id = to_abi_compatible_number(self.authorized_msa_id);
		let intent_ids: Vec<u8> = self
			.intent_ids
			.iter()
			.flat_map(|intent_id| to_abi_compatible_number(*intent_id))
			.collect();
		let intent_ids = sp_io::hashing::keccak_256(&intent_ids);
		let coded_expiration = to_abi_compatible_number(self.expiration);
		let message = sp_io::hashing::keccak_256(
			&[MAIN_TYPE_HASH.as_slice(), &coded_authorized_msa_id, &intent_ids, &coded_expiration]
				.concat(),
		);
		let combined = [prefix_domain_separator.as_ref(), &message].concat();
		combined.into_boxed_slice()
	}
}

impl AddProvider {
	/// Create new `AddProvider`
	pub fn new(
		authorized_msa_id: MessageSourceId,
		intent_ids: Option<Vec<IntentId>>,
		expiration: BlockNumber,
	) -> Self {
		let intent_ids = intent_ids.unwrap_or_default();

		Self { authorized_msa_id, intent_ids, expiration }
	}
}

/// A type definition for hash types used in the MSA Recovery System.
pub type RecoveryHash = [u8; 32]; // 32 bytes for

/// A type definition for the Recovery Commitment data for the following operation:
/// -  Adding a Recovery Commitment - `pallet_msa::add_recovery_commitment
pub type RecoveryCommitment = RecoveryHash; // 32 bytes for the recovery commitment

/// A type definition for the payload for the Recovery Commitment operation:
/// -  Adding a Recovery Commitment - `pallet_msa::add_recovery_commitment`
#[derive(
	TypeInfo, RuntimeDebugNoBound, Clone, Decode, DecodeWithMemTracking, Encode, PartialEq, Eq,
)]
#[scale_info(skip_type_params(T))]
pub struct RecoveryCommitmentPayload<T: Config> {
	/// type discriminator
	pub discriminant: PayloadTypeDiscriminator,
	/// The Recovery Commitment (32 bytes)
	pub recovery_commitment: RecoveryCommitment,
	/// The block number at which a signed proof of this payload expires.
	pub expiration: BlockNumberFor<T>,
}

impl<T: Config> EIP712Encode for RecoveryCommitmentPayload<T> {
	fn encode_eip_712(&self, chain_id: u32) -> Box<[u8]> {
		lazy_static! {
			// signed payload
			static ref MAIN_TYPE_HASH: [u8; 32] = sp_io::hashing::keccak_256(
				b"RecoveryCommitmentPayload(bytes recoveryCommitment,uint32 expiration)",
			);
		}
		// get prefix and domain separator
		let prefix_domain_separator: Box<[u8]> =
			get_eip712_encoding_prefix("0xcccccccccccccccccccccccccccccccccccccccc", chain_id);
		let hashed_recovery = sp_io::hashing::keccak_256(&self.recovery_commitment);
		let expiration: U256 = self.expiration.into();
		let coded_expiration = to_abi_compatible_number(expiration.as_u128());
		let message = sp_io::hashing::keccak_256(
			&[MAIN_TYPE_HASH.as_slice(), hashed_recovery.as_slice(), &coded_expiration].concat(),
		);
		let combined = [prefix_domain_separator.as_ref(), &message].concat();
		combined.into_boxed_slice()
	}
}

/// The interface for mutating Intent permissions in a delegation relationship.
pub trait PermittedDelegationIntents<T: Config> {
	/// Attempt to insert a new Intent. Dispatches error when the max allowed delegations are exceeded.
	fn try_insert_intent(&mut self, intent_id: IntentId) -> Result<(), DispatchError>;

	/// Attempt to insert a collection of Intents. Dispatches error when the max allowed delegations are exceeded.
	fn try_insert_intents(&mut self, intent_ids: Vec<IntentId>) -> Result<(), DispatchError> {
		for intent_id in intent_ids.into_iter() {
			self.try_insert_intent(intent_id)?;
		}

		Ok(())
	}

	/// Attempt get and mutate a collection of Intents. Dispatches error when an Intent cannot be found.
	fn try_get_mut_intents(
		&mut self,
		intent_ids: Vec<IntentId>,
		block_number: BlockNumberFor<T>,
	) -> Result<(), DispatchError> {
		for intent_id in intent_ids.into_iter() {
			self.try_get_mut_intent(intent_id, block_number)?;
		}
		Ok(())
	}

	/// Attempt get and mutate an Intent. Dispatches error when an Intent cannot be found.
	fn try_get_mut_intent(
		&mut self,
		intent_id: IntentId,
		block_number: BlockNumberFor<T>,
	) -> Result<(), DispatchError>;
}

/// Implementation of SchemaPermission trait on Delegation type.
impl<T: Config> PermittedDelegationIntents<T>
	for Delegation<IntentId, BlockNumberFor<T>, T::MaxGrantsPerDelegation>
{
	/// Attempt to insert a new schema. Dispatches error when the max allowed schemas are exceeded.
	fn try_insert_intent(&mut self, intent_id: IntentId) -> Result<(), DispatchError> {
		self.permissions
			.try_insert(intent_id, Default::default())
			.map_err(|_| Error::<T>::ExceedsMaxGrantsPerDelegation)?;
		Ok(())
	}

	/// Attempt get and mutate a schema. Dispatches error when a schema cannot be found.
	fn try_get_mut_intent(
		&mut self,
		intent_id: IntentId,
		block_number: BlockNumberFor<T>,
	) -> Result<(), DispatchError> {
		let intent =
			self.permissions.get_mut(&intent_id).ok_or(Error::<T>::PermissionNotGranted)?;

		*intent = block_number;

		Ok(())
	}
}

/// Helper function to compute CID of given bytes and return Vec<u8>
#[cfg(any(test, feature = "runtime-benchmarks"))]
pub fn compute_cid(bytes: &[u8]) -> Vec<u8> {
	let cid = compute_cid_v1(bytes).expect("Failed to compute CID");
	// Encode CID into multibase Base58btc string, then return as Vec<u8>
	let encoded = multibase::encode(multibase::Base::Base58Btc, cid);
	encoded.into_bytes()
}

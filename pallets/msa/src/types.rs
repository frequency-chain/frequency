//! Types for the MSA Pallet
#![cfg_attr(not(feature = "std"), no_std)]

use super::*;
use parity_scale_codec::{Decode, Encode};

use core::fmt::Debug;

pub use common_primitives::msa::{
	Delegation, DelegatorId, KeyInfoResponse, MessageSourceId, ProviderId,
};
use common_primitives::{node::BlockNumber, schema::SchemaId};

use common_primitives::{
	signatures::{get_eip712_encoding_prefix, AccountAddressMapper, EthereumAddressMapper},
	utils::to_abi_compatible_number,
};
use scale_info::TypeInfo;
use sp_core::U256;

/// Dispatch Empty
pub const EMPTY_FUNCTION: fn(MessageSourceId) -> DispatchResult = |_| Ok(());

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
	pub schema_ids: Vec<SchemaId>,
	/// The block number at which the proof for grant_delegation expires.
	pub expiration: BlockNumber,
}

impl EIP712Encode for AddProvider {
	fn encode_eip_712(&self, chain_id: u32) -> Box<[u8]> {
		lazy_static! {
			// signed payload
			static ref MAIN_TYPE_HASH: [u8; 32] = sp_io::hashing::keccak_256(
				b"AddProvider(uint64 authorizedMsaId,uint16[] schemaIds,uint32 expiration)"
			);
		}
		// get prefix and domain separator
		let prefix_domain_separator: Box<[u8]> =
			get_eip712_encoding_prefix("0xcccccccccccccccccccccccccccccccccccccccc", chain_id);
		let coded_authorized_msa_id = to_abi_compatible_number(self.authorized_msa_id);
		let schema_ids: Vec<u8> = self
			.schema_ids
			.iter()
			.flat_map(|schema_id| to_abi_compatible_number(*schema_id))
			.collect();
		let schema_ids = sp_io::hashing::keccak_256(&schema_ids);
		let coded_expiration = to_abi_compatible_number(self.expiration);
		let message = sp_io::hashing::keccak_256(
			&[MAIN_TYPE_HASH.as_slice(), &coded_authorized_msa_id, &schema_ids, &coded_expiration]
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
		schema_ids: Option<Vec<SchemaId>>,
		expiration: BlockNumber,
	) -> Self {
		let schema_ids = schema_ids.unwrap_or_default();

		Self { authorized_msa_id, schema_ids, expiration }
	}
}

/// A type definition for the Recovery Commitment data for the following operation:
/// -  Adding a Recovery Commitment - `pallet_msa::add_recovery_commitment
pub type RecoveryCommitment = [u8; 32]; // 32 bytes for the recovery commitment

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
				b"RecoveryCommitmentPayload(uint64 msaId,uint32 expiration,bytes32 recoveryCommitment)",
			);
		}
		// get prefix and domain separator
		let prefix_domain_separator: Box<[u8]> =
			get_eip712_encoding_prefix("0xcccccccccccccccccccccccccccccccccccccccc", chain_id);
		let expiration: U256 = self.expiration.into();
		let coded_expiration = to_abi_compatible_number(expiration.as_u128());
		let message = sp_io::hashing::keccak_256(
			&[MAIN_TYPE_HASH.as_slice(), &coded_expiration, &self.recovery_commitment].concat(),
		);
		let combined = [prefix_domain_separator.as_ref(), &message].concat();
		combined.into_boxed_slice()
	}
}

/// The interface for mutating schemas permissions in a delegation relationship.
pub trait PermittedDelegationSchemas<T: Config> {
	/// Attempt to insert a new schema. Dispatches error when the max allowed schemas are exceeded.
	fn try_insert_schema(&mut self, schema_id: SchemaId) -> Result<(), DispatchError>;

	/// Attempt to insert a collection of schemas. Dispatches error when the max allowed schemas are exceeded.
	fn try_insert_schemas(&mut self, schema_ids: Vec<SchemaId>) -> Result<(), DispatchError> {
		for schema_id in schema_ids.into_iter() {
			self.try_insert_schema(schema_id)?;
		}

		Ok(())
	}

	/// Attempt get and mutate a collection of schemas. Dispatches error when a schema cannot be found.
	fn try_get_mut_schemas(
		&mut self,
		schema_ids: Vec<SchemaId>,
		block_number: BlockNumberFor<T>,
	) -> Result<(), DispatchError> {
		for schema_id in schema_ids.into_iter() {
			self.try_get_mut_schema(schema_id, block_number)?;
		}
		Ok(())
	}

	/// Attempt get and mutate a schema. Dispatches error when a schema cannot be found.
	fn try_get_mut_schema(
		&mut self,
		schema_id: SchemaId,
		block_number: BlockNumberFor<T>,
	) -> Result<(), DispatchError>;
}

/// Implementation of SchemaPermission trait on Delegation type.
impl<T: Config> PermittedDelegationSchemas<T>
	for Delegation<SchemaId, BlockNumberFor<T>, T::MaxSchemaGrantsPerDelegation>
{
	/// Attempt to insert a new schema. Dispatches error when the max allowed schemas are exceeded.
	fn try_insert_schema(&mut self, schema_id: SchemaId) -> Result<(), DispatchError> {
		self.schema_permissions
			.try_insert(schema_id, Default::default())
			.map_err(|_| Error::<T>::ExceedsMaxSchemaGrantsPerDelegation)?;
		Ok(())
	}

	/// Attempt get and mutate a schema. Dispatches error when a schema cannot be found.
	fn try_get_mut_schema(
		&mut self,
		schema_id: SchemaId,
		block_number: BlockNumberFor<T>,
	) -> Result<(), DispatchError> {
		let schema = self
			.schema_permissions
			.get_mut(&schema_id)
			.ok_or(Error::<T>::SchemaNotGranted)?;

		*schema = block_number;

		Ok(())
	}
}

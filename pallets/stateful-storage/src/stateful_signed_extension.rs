//! Substrate Signed Extension for validating requests to the stateful-storage pallet
use crate::{types::ItemAction, Call, Config, Pallet};
use core::marker::PhantomData;

use codec::{Decode, Encode};
use common_primitives::{
	msa::MessageSourceId,
	schema::{PayloadLocation, SchemaId},
	stateful_storage::{PageHash, PageId},
};
use frame_support::{
	dispatch::{DispatchInfo, Dispatchable},
	pallet_prelude::ValidTransaction,
	traits::IsSubType,
	BoundedVec,
};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{DispatchInfoOf, SignedExtension},
	transaction_validity::{InvalidTransaction, TransactionValidity, TransactionValidityError},
	DispatchError,
};

/// The SignedExtension trait is implemented on CheckFreeExtrinsicUse to validate that a provider
/// has not already been revoked if the calling extrinsic is revoking a provider to an MSA. The
/// purpose of this is to ensure that the revoke_delegation_by_delegator extrinsic cannot be
/// repeatedly called and flood the network.
#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct StatefulSignedExtension<T: Config + Send + Sync>(PhantomData<T>);

/// Validator trait to be used in validating Stateful Storage
/// requests from a Signed Extension.
pub trait StatefulStorageValidate<C: Config> {
	/// Validates the parameters to an Itemized extrinsic call
	fn validate_itemized(
		msa_id: &MessageSourceId,
		schema_id: &SchemaId,
		actions: &BoundedVec<ItemAction<C::MaxItemizedBlobSizeBytes>, C::MaxItemizedActionsCount>,
		is_payload_signed: bool,
		target_hash: &PageHash,
	) -> TransactionValidity;

	/// Validates the parameters to a Paginated extrinsic call
	fn validate_paginated(
		msa_id: &MessageSourceId,
		schema_id: &SchemaId,
		page_id: &PageId,
		is_payload_signed: bool,
		page_delete: bool,
		target_hash: &PageHash,
	) -> TransactionValidity;
}

impl<T: Config> StatefulStorageValidate<T> for Pallet<T> {
	/// Validates the following criteria for an Itemized Stateful Storage extrinsic:
	///
	/// * schema_id represents a valid schema
	/// * schema `PayloadLocation` is `Paginated`
	/// * schema allows signed payloads if `is_payload_signed` == true
	/// * schema is not `AppendOnly` if `page_delete` == true
	/// * page can be read/parsed
	/// * known page state hash matches provided target hash
	///
	/// Returns a `ValidTransaction` or wrapped [`pallet::Error`]
	///
	/// # Arguments:
	/// * account_id: the account id associated with the MSA owning the storage
	/// * msa_id: the `MessageSourceAccount`
	/// * schema_id: the `SchemaId` for the Paginated storage
	/// * actions: Array of `ItemAction` representing adds/deletes to Itemized storage
	/// * is_payload_signed: whether we were called with a signed payload
	/// * target_hash: the content hash representing the known state of the Paginated storage.
	///
	/// # Errors (as u8 wrapped by `InvalidTransaction::Custom`)
	/// * [`Error::InvalidSchemaId`]
	/// * [`Error::SchemaPayloadLocationMismatch`]
	/// * [`Error::SchemaNotSupported`]
	/// * [`Error::CorruptedState`]
	/// * [`Error::StalePageState`]
	///
	fn validate_itemized(
		msa_id: &MessageSourceId,
		schema_id: &SchemaId,
		actions: &BoundedVec<ItemAction<T::MaxItemizedBlobSizeBytes>, T::MaxItemizedActionsCount>,
		is_payload_signed: bool,
		target_hash: &PageHash,
	) -> TransactionValidity {
		const TAG_PREFIX: &str = "StatefulStorageItemized";
		let is_pruning = actions.iter().any(|a| matches!(a, ItemAction::Delete { .. }));

		Self::check_schema_for_write(
			*schema_id,
			PayloadLocation::Itemized,
			is_payload_signed,
			is_pruning,
		)
		.map_err(|e| map_dispatch_error(e))?;

		Self::get_checked_itemized_page(msa_id, schema_id, target_hash)
			.map_err(|e| map_dispatch_error(e))?;

		return ValidTransaction::with_tag_prefix(TAG_PREFIX)
			.and_provides((msa_id, schema_id))
			.build()
	}

	/// Validates the following criteria for a Paginated Stateful Storage extrinsic:
	///
	/// * schema_id represents a valid schema
	/// * schema `PayloadLocation` is `Paginated`
	/// * schema allows signed payloads if `is_payload_signed` == true
	/// * schema is not `AppendOnly` if `page_delete` == true
	/// * page can be read
	/// * known page state hash matches provided target hash
	///
	/// Returns a `ValidTransaction` or wrapped [`pallet::Error`]
	///
	/// # Arguments:
	/// * account_id: the account id associated with the MSA owning the storage
	/// * msa_id: the `MessageSourceAccount`
	/// * schema_id: the `SchemaId` for the Paginated storage
	/// * page_id: the `PageId` to be validated
	/// * is_payload_signed: whether we were called with a signed payload
	/// * page_delete: whether the invoking operation is a page deletion
	/// * target_hash: the content hash representing the known state of the Paginated storage.
	///
	/// # Errors (as u8 wrapped by `InvalidTransaction::Custom`)
	/// * [`Error::InvalidSchemaId`]
	/// * [`Error::SchemaPayloadLocationMismatch`]
	/// * [`Error::SchemaNotSupported`]
	/// * [`Error::CorruptedState`]
	/// * [`Error::StalePageState`]
	///
	fn validate_paginated(
		msa_id: &MessageSourceId,
		schema_id: &SchemaId,
		page_id: &PageId,
		is_payload_signed: bool,
		page_delete: bool,
		target_hash: &PageHash,
	) -> TransactionValidity {
		const TAG_PREFIX: &str = "StatefulStoragePaginated";

		Self::check_schema_for_write(
			*schema_id,
			PayloadLocation::Paginated,
			is_payload_signed,
			page_delete,
		)
		.map_err(|e| map_dispatch_error(e))?;

		Self::check_paginated_state(msa_id, schema_id, page_id, target_hash)
			.map_err(|e| map_dispatch_error(e))?;

		return ValidTransaction::with_tag_prefix(TAG_PREFIX)
			.and_provides((msa_id, schema_id, page_id))
			.build()
	}
}

/// Map a module DispatchError to an InvalidTransaction::Custom error
pub fn map_dispatch_error(err: DispatchError) -> InvalidTransaction {
	InvalidTransaction::Custom(match err {
		DispatchError::Module(module_err) => module_err.index,
		_ => 255u8,
	})
}

impl<T: Config + Send + Sync> StatefulSignedExtension<T> {
	/// Create new `SignedExtension`.
	pub fn new() -> Self {
		Self(sp_std::marker::PhantomData)
	}
}

impl<T: Config + Send + Sync> sp_std::fmt::Debug for StatefulSignedExtension<T> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "StatefulSignedExtension<{:?}>", self.0)
	}
	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

impl<T: Config + Send + Sync> SignedExtension for StatefulSignedExtension<T>
where
	T::RuntimeCall: Dispatchable<Info = DispatchInfo> + IsSubType<Call<T>>,
{
	type AccountId = T::AccountId;
	type Call = T::RuntimeCall;
	type AdditionalSigned = ();
	type Pre = ();
	const IDENTIFIER: &'static str = "StatefulSignedExtension";

	fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
		Ok(())
	}

	fn pre_dispatch(
		self,
		who: &Self::AccountId,
		call: &Self::Call,
		info: &DispatchInfoOf<Self::Call>,
		len: usize,
	) -> Result<Self::Pre, TransactionValidityError> {
		self.validate(who, call, info, len).map(|_| ())
	}

	/// Frequently called by the transaction queue to validate all free MSA extrinsics:
	/// Returns a `ValidTransaction` or wrapped [`ValidityError`]
	/// * revoke_delegation_by_provider
	/// * revoke_delegation_by_delegator
	/// * delete_msa_public_key
	/// * retire_msa
	/// Validate functions for the above MUST prevent errors in the extrinsic logic to prevent spam.
	///
	/// Arguments:
	/// who: AccountId calling the extrinsic
	/// call: The pallet extrinsic being called
	/// unused: _info, _len
	///
	fn validate(
		&self,
		_who: &Self::AccountId,
		call: &Self::Call,
		_info: &DispatchInfoOf<Self::Call>,
		_len: usize,
	) -> TransactionValidity {
		match call.is_sub_type() {
			Some(Call::apply_item_actions {
				state_owner_msa_id,
				schema_id,
				actions,
				target_hash,
				..
			}) => Pallet::<T>::validate_itemized(
				state_owner_msa_id,
				schema_id,
				actions,
				false,
				target_hash,
			),
			Some(Call::apply_item_actions_with_signature { payload, .. }) =>
				Pallet::<T>::validate_itemized(
					&payload.msa_id,
					&payload.schema_id,
					&payload.actions,
					true,
					&payload.target_hash,
				),
			Some(Call::upsert_page {
				state_owner_msa_id, schema_id, page_id, target_hash, ..
			}) => Pallet::<T>::validate_paginated(
				state_owner_msa_id,
				schema_id,
				page_id,
				false,
				false,
				target_hash,
			),
			Some(Call::upsert_page_with_signature { payload, .. }) =>
				Pallet::<T>::validate_paginated(
					&payload.msa_id,
					&payload.schema_id,
					&payload.page_id,
					true,
					false,
					&payload.target_hash,
				),
			Some(Call::delete_page { state_owner_msa_id, schema_id, page_id, target_hash }) =>
				Pallet::<T>::validate_paginated(
					state_owner_msa_id,
					schema_id,
					page_id,
					false,
					true,
					target_hash,
				),
			Some(Call::delete_page_with_signature { payload, .. }) =>
				Pallet::<T>::validate_paginated(
					&payload.msa_id,
					&payload.schema_id,
					&payload.page_id,
					true,
					true,
					&payload.target_hash,
				),
			_ => Ok(Default::default()),
		}
	}
}

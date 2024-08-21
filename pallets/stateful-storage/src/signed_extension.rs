//! Substrate Signed Extension for validating requests to the stateful storage pallet
use crate::{Call, Config, Error, Pallet};
use common_primitives::{
	msa::{MessageSourceId, MsaValidator, SchemaId},
	stateful_storage::{PageHash, PageId},
};
use core::marker::PhantomData;
use frame_support::{
	dispatch::DispatchInfo, ensure, pallet_prelude::ValidTransaction, traits::IsSubType,
};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{DispatchInfoOf, Dispatchable, SignedExtension},
	transaction_validity::{InvalidTransaction, TransactionValidity, TransactionValidityError},
	DispatchError,
};

/// The SignedExtension trait is implemented on CheckFreeExtrinsicUse to validate the request. The
/// purpose of this is to ensure that the target_hash is verified in transaction pool before getting
/// into block. This is to reduce the chance that capacity consumption due to stale hash
#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct StatefulStorageSignedExtension<T: Config + Send + Sync>(PhantomData<T>);

impl<T: Config + Send + Sync> StatefulStorageSignedExtension<T> {
	/// Create new `SignedExtension`.
	pub fn new() -> Self {
		Self(sp_std::marker::PhantomData)
	}

	/// Verifies the hashes for an Itemized Stateful Storage extrinsic
	fn verify_hash_itemized(
		msa_id: &MessageSourceId,
		schema_id: &SchemaId,
		target_hash: &PageHash,
	) -> TransactionValidity {
		const TAG_PREFIX: &str = "StatefulStorageHashItemized";

		if let Ok(Some(page)) = Pallet::<T>::get_itemized_page_for(*msa_id, *schema_id) {
			let current_hash: PageHash = page.get_hash();

			ensure!(
				&current_hash == target_hash,
				map_dispatch_error(Error::<T>::StalePageState.into())
			);

			return ValidTransaction::with_tag_prefix(TAG_PREFIX)
				.and_provides((msa_id, schema_id))
				.build()
		}
		Ok(Default::default())
	}

	/// Verifies the hashes for a Paginated Stateful Storage extrinsic
	fn verify_hash_paginated(
		msa_id: &MessageSourceId,
		schema_id: &SchemaId,
		page_id: &PageId,
		target_hash: &PageHash,
	) -> TransactionValidity {
		const TAG_PREFIX: &str = "StatefulStorageHashPaginated";
		if let Ok(Some(page)) = Pallet::<T>::get_paginated_page_for(*msa_id, *schema_id, *page_id) {
			let current_hash: PageHash = page.get_hash();

			ensure!(
				&current_hash == target_hash,
				map_dispatch_error(Error::<T>::StalePageState.into())
			);

			return ValidTransaction::with_tag_prefix(TAG_PREFIX)
				.and_provides((msa_id, schema_id, page_id))
				.build()
		}

		Ok(Default::default())
	}
}

impl<T: Config + Send + Sync> sp_std::fmt::Debug for StatefulStorageSignedExtension<T> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "StatefulStorageSignedExtension<{:?}>", self.0)
	}
	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

/// Map a module DispatchError to an InvalidTransaction::Custom error
pub fn map_dispatch_error(err: DispatchError) -> InvalidTransaction {
	InvalidTransaction::Custom(match err {
		DispatchError::Module(module_err) =>
			<u32 as Decode>::decode(&mut module_err.error.as_slice())
				.unwrap_or_default()
				.try_into()
				.unwrap_or_default(),
		_ => 255u8,
	})
}

impl<T: Config + Send + Sync> SignedExtension for StatefulStorageSignedExtension<T>
where
	T::RuntimeCall: Dispatchable<Info = DispatchInfo> + IsSubType<Call<T>>,
{
	type AccountId = T::AccountId;
	type Call = T::RuntimeCall;
	type AdditionalSigned = ();
	type Pre = ();
	const IDENTIFIER: &'static str = "StatefulStorageSignedExtension";

	/// Additional signed
	fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
		Ok(())
	}

	/// Pre dispatch
	fn pre_dispatch(
		self,
		_who: &Self::AccountId,
		_call: &Self::Call,
		_info: &DispatchInfoOf<Self::Call>,
		_len: usize,
	) -> Result<Self::Pre, TransactionValidityError> {
		// Since we already check the hash in stateful-storage-pallet extrinsics we do not need to
		// check the hash before dispatching
		Ok(())
	}

	/// Filters stateful extrinsics and verifies the hashes to ensure we are proceeding
	/// with a stale one
	fn validate(
		&self,
		_who: &Self::AccountId,
		call: &Self::Call,
		_info: &DispatchInfoOf<Self::Call>,
		_len: usize,
	) -> TransactionValidity {
		match call.is_sub_type() {
			Some(Call::apply_item_actions {
				state_owner_msa_id, schema_id, target_hash, ..
			}) => Self::verify_hash_itemized(state_owner_msa_id, schema_id, target_hash),
			Some(Call::upsert_page {
				state_owner_msa_id, schema_id, target_hash, page_id, ..
			}) => Self::verify_hash_paginated(state_owner_msa_id, schema_id, page_id, target_hash),
			Some(Call::delete_page {
				state_owner_msa_id, schema_id, target_hash, page_id, ..
			}) => Self::verify_hash_paginated(state_owner_msa_id, schema_id, page_id, target_hash),
			Some(Call::apply_item_actions_with_signature { payload, .. }) =>
				Self::verify_hash_itemized(
					&payload.msa_id,
					&payload.schema_id,
					&payload.target_hash,
				),
			Some(Call::upsert_page_with_signature { payload, .. }) => Self::verify_hash_paginated(
				&payload.msa_id,
				&payload.schema_id,
				&payload.page_id,
				&payload.target_hash,
			),
			Some(Call::delete_page_with_signature { payload, .. }) => Self::verify_hash_paginated(
				&payload.msa_id,
				&payload.schema_id,
				&payload.page_id,
				&payload.target_hash,
			),
			Some(Call::apply_item_actions_with_signature_v2 { payload, delegator_key, .. }) => {
				let state_owner_msa_id = T::MsaInfoProvider::ensure_valid_msa_key(delegator_key)
					.map_err(|e| map_dispatch_error(e))?;
				Self::verify_hash_itemized(
					&state_owner_msa_id,
					&payload.schema_id,
					&payload.target_hash,
				)
			},
			Some(Call::upsert_page_with_signature_v2 { payload, delegator_key, .. }) => {
				let state_owner_msa_id = T::MsaInfoProvider::ensure_valid_msa_key(delegator_key)
					.map_err(|e| map_dispatch_error(e))?;
				Self::verify_hash_paginated(
					&state_owner_msa_id,
					&payload.schema_id,
					&payload.page_id,
					&payload.target_hash,
				)
			},
			Some(Call::delete_page_with_signature_v2 { payload, delegator_key, .. }) => {
				let state_owner_msa_id = T::MsaInfoProvider::ensure_valid_msa_key(delegator_key)
					.map_err(|e| map_dispatch_error(e))?;
				Self::verify_hash_paginated(
					&state_owner_msa_id,
					&payload.schema_id,
					&payload.page_id,
					&payload.target_hash,
				)
			},
			_ => Ok(Default::default()),
		}
	}
}

//! Substrate Signed Extension for validating requests to the stateful storage pallet
use crate::{
	stateful_child_tree::StatefulChildTree,
	types::{
		ItemizedKey, ItemizedPage, PaginatedKey, PaginatedPage, ITEMIZED_STORAGE_PREFIX,
		PAGINATED_STORAGE_PREFIX, PALLET_STORAGE_PREFIX,
	},
	Call, Config, Error,
};
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
	DispatchError, ModuleError,
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

	fn verify_hash(
		msa_id: &MessageSourceId,
		schema_id: &SchemaId,
		page_id_option: Option<&PageId>,
		target_hash: &PageHash,
	) -> TransactionValidity {
		const TAG_PREFIX: &str = "StatefulStorageHash";
		let current_hash: PageHash = match page_id_option {
			Some(page_id) => {
				// paginated page
				let keys: PaginatedKey = (*schema_id, *page_id);
				match StatefulChildTree::<T::KeyHasher>::try_read::<_, PaginatedPage<T>>(
					&msa_id,
					PALLET_STORAGE_PREFIX,
					PAGINATED_STORAGE_PREFIX,
					&keys,
				) {
					Ok(Some(page)) => page.get_hash(),
					_ => PageHash::default(),
				}
			},
			None => {
				// Itemized pages don't have a page id
				let keys: ItemizedKey = (*schema_id,);
				match StatefulChildTree::<T::KeyHasher>::try_read::<_, ItemizedPage<T>>(
					&msa_id,
					PALLET_STORAGE_PREFIX,
					ITEMIZED_STORAGE_PREFIX,
					&keys,
				) {
					Ok(Some(page)) => page.get_hash(),
					_ => PageHash::default(),
				}
			},
		};

		ensure!(
			&current_hash == target_hash,
			map_dispatch_error(DispatchError::Other(Error::<T>::StalePageState.into()))
		);

		let mut transaction = ValidTransaction::with_tag_prefix(TAG_PREFIX);
		if let Some(page_id) = page_id_option {
			transaction = transaction.and_provides((msa_id, schema_id, page_id));
		} else {
			transaction = transaction.and_provides((msa_id, schema_id));
		}
		return transaction.build();
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
	log::debug!(
		target: "runtime::StatefulStorageSignedExtension",
		"{:?}",
		&err,
	);
	InvalidTransaction::Custom(match err {
		DispatchError::Module(ModuleError { error, .. }) => error[0],
		DispatchError::Other(_) => 250u8, // stale page
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
		Ok(())
	}

	/// Frequently called by the transaction queue to validate all free Handles extrinsics:
	/// Returns a `ValidTransaction` or wrapped [`TransactionValidityError`]
	/// * retire_handle
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
				state_owner_msa_id, schema_id, target_hash, ..
			}) => Self::verify_hash(state_owner_msa_id, schema_id, None, target_hash),
			Some(Call::upsert_page {
				state_owner_msa_id, schema_id, target_hash, page_id, ..
			}) => Self::verify_hash(state_owner_msa_id, schema_id, Some(page_id), target_hash),
			Some(Call::delete_page {
				state_owner_msa_id, schema_id, target_hash, page_id, ..
			}) => Self::verify_hash(state_owner_msa_id, schema_id, Some(page_id), target_hash),
			Some(Call::apply_item_actions_with_signature { payload, .. }) =>
				Self::verify_hash(&payload.msa_id, &payload.schema_id, None, &payload.target_hash),
			Some(Call::upsert_page_with_signature { payload, .. }) => Self::verify_hash(
				&payload.msa_id,
				&payload.schema_id,
				Some(&payload.page_id),
				&payload.target_hash,
			),
			Some(Call::delete_page_with_signature { payload, .. }) => Self::verify_hash(
				&payload.msa_id,
				&payload.schema_id,
				Some(&payload.page_id),
				&payload.target_hash,
			),
			Some(Call::apply_item_actions_with_signature_v2 { payload, delegator_key, .. }) => {
				let state_owner_msa_id = T::MsaInfoProvider::ensure_valid_msa_key(delegator_key)
					.map_err(|e| map_dispatch_error(e))?;
				Self::verify_hash(
					&state_owner_msa_id,
					&payload.schema_id,
					None,
					&payload.target_hash,
				)
			},
			Some(Call::upsert_page_with_signature_v2 { payload, delegator_key, .. }) => {
				let state_owner_msa_id = T::MsaInfoProvider::ensure_valid_msa_key(delegator_key)
					.map_err(|e| map_dispatch_error(e))?;
				Self::verify_hash(
					&state_owner_msa_id,
					&payload.schema_id,
					Some(&payload.page_id),
					&payload.target_hash,
				)
			},
			Some(Call::delete_page_with_signature_v2 { payload, delegator_key, .. }) => {
				let state_owner_msa_id = T::MsaInfoProvider::ensure_valid_msa_key(delegator_key)
					.map_err(|e| map_dispatch_error(e))?;
				Self::verify_hash(
					&state_owner_msa_id,
					&payload.schema_id,
					Some(&payload.page_id),
					&payload.target_hash,
				)
			},
			_ => Ok(Default::default()),
		}
	}
}

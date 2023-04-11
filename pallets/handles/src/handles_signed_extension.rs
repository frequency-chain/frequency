//! Substrate Signed Extension for validating requests to the handles pallet
use crate::{Call, Config, Pallet};
use codec::{Decode, Encode};
use common_primitives::msa::MessageSourceId;
use core::marker::PhantomData;
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

#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct HandlesSignedExtension<T: Config + Send + Sync>(PhantomData<T>);

impl<T: Config + Send + Sync> HandlesSignedExtension<T> {
	/// Create new `SignedExtension`.
	pub fn new() -> Self {
		Self(sp_std::marker::PhantomData)
	}
}

/// Validator trait to be used in validating Handles requests from a Signed Extension.
pub trait HandlesValidate<C: Config> {
	/// Validates calls to the retire_handle() extrinsic
	fn validate_retire_handle() -> TransactionValidity;
}

impl<T: Config> HandlesValidate<T> for Pallet<T> {
	/// # Errors (as u8 wrapped by `InvalidTransaction::Custom`)
	/// * [`Error::InvalidMessageSourceAccount`]
	/// * [`Error::MSAHandleDoesNotExist`]
	fn validate_retire_handle() -> TransactionValidity {
		const TAG_PREFIX: &str = "HandlesRetireHandle";
		// Validation: The delegator must already have a MSA id
		let delegator_msa_id = T::MsaInfoProvider::ensure_valid_msa_key(&delegator_key)
			.map_err(|_| Error::<T>::InvalidMessageSourceAccount)?;

		let display_name_handle = MSAIdToDisplayName::<T>::try_get(delegator_msa_id)
			.map_err(|_| Error::<T>::MSAHandleDoesNotExist)?;
	}
}

impl<T: Config + Send + Sync> SignedExtension for HandlesSignedExtension<T>
where
	T::RuntimeCall: Dispatchable<Info = DispatchInfo> + IsSubType<Call<T>>,
{
	type AccountId = T::AccountId;
	type Call = T::RuntimeCall;
	type AdditionalSigned = ();
	type Pre = ();
	const IDENTIFIER: &'static str = "HandlesSignedExtension";

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

	/// Frequently called by the transaction queue to validate all free Handles extrinsics:
	/// Returns a `ValidTransaction` or wrapped [`ValidityError`]
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
			Some(Call::retire_handle) => Pallet::<T>::validate_retire_handle(),
			_ => Ok(Default::default()),
		}
	}
}
